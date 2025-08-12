//! Manages the current `TradingView` session
//! allows for the receiving of data and the defining of protocols
use std::collections::hash_map;
use std::collections::HashMap;

use crate::protocol::{
    format_ws_ping, into_inner_identifier, parse_ws_packet, IntoWSVecValues, Packet, WSPacket,
};
use crate::utils::generate_session_id;
use futures_util::stream::SplitStream;

use tokio::sync::mpsc;

use tokio::sync::mpsc::Sender;
use tokio_tungstenite::{
    connect_async, tungstenite::client::IntoClientRequest, tungstenite::Message, MaybeTlsStream,
    WebSocketStream,
};

use futures_util::future::BoxFuture;

use futures_util::{stream::SplitSink, SinkExt, StreamExt};

use tokio::net::TcpStream;

const CONNECTION: &str = "wss://data.tradingview.com/socket.io/websocket";

/// The two possible field types that can be used for data retrieval:
/// - All = all available `TradingView` fields/datapoints
/// - Price = only fields/datapoints related to price
#[allow(dead_code)]
#[derive(Debug, PartialEq)]
enum FieldTypes {
    All,
    Price,
}

#[macro_use]
mod message_processors {
    macro_rules! convert_to_message_processor {
        ($f:expr) => {
            |message: &Packet<'_>, tx_to_send| Box::pin($f(message, tx_to_send))
        };
    }
}

/// The data related to a particular symbol
///
/// # Arguments
///
/// * `symbol`: The specified symbol that data is being collected for, in format `MARKET:SYMBOL` e.g., `NYSE:AAPL`
/// * `price`: A tokio mpsc sender stream, used for sending messages to the server
/// * `technical_analysis`: The current data from the datastream about prices and technical analysis, set by either `set_data_price` or `set_data_ta`
#[derive(Debug, Clone)]
pub struct SymbolData {
    pub symbol: String,
    pub price: f64,
    pub technical_analysis: f64,
}

/// All the possible fields for a `TradingView` session, impacts what is received
const FIELDS: [&str; 48] = [
    "base-currency-logoid",
    "ch",
    "chp",
    "currency-logoid",
    "currency_code",
    "current_session",
    "description",
    "exchange",
    "format",
    "fractional",
    "is_tradable",
    "language",
    "local_description",
    "logoid",
    "lp",
    "lp_time",
    "minmov",
    "minmove2",
    "original_name",
    "pricescale",
    "pro_name",
    "short_name",
    "type",
    "update_mode",
    "volume",
    "ask",
    "bid",
    "fundamentals",
    "high_price",
    "low_price",
    "open_price",
    "prev_close_price",
    "rch",
    "rchp",
    "rtc",
    "rtc_time",
    "status",
    "industry",
    "basic_eps_net_income",
    "beta_1_year",
    "market_cap_basic",
    "earnings_per_share_basic_ttm",
    "price_earnings_ttm",
    "sector",
    "dividends_yield",
    "timezone",
    "country_code",
    "provider_id",
];

/// A session which encapsulates the current state of the `TradingView` session.
///
/// This session holds the id, the sending mpsc socket and the data that is incoming.
///
/// # Fields
///
/// * `session_id`: The current id of the session, used to authenticate with `TradingView`
/// * `tx_to_send`: A tokio mpsc sender stream, used for sending messages to the server
/// * `data`: A hashmap of the current data from the datastream about prices and technical analysis, set by either '`set_data_price`' or '`set_data_ta`'
/// * `rx_to_send`: An optional tokio mpsc receiver stream, used for receiving messages from the server
/// * `read`: An optional tokio `WebSocket` stream, used for reading messages from the server
/// * `processors`: A vector of message processors, used for processing incoming messages from the server
/// * `chart_details`: An optional `ChartSession` struct containing the current state of the `TradingView` chart session
pub struct Session {
    pub session_id: String,
    pub tx_to_send: mpsc::Sender<String>,
    data: HashMap<String, (f64, f64)>,
    rx_to_send: Option<mpsc::Receiver<String>>,
    processors: Vec<MessageProcessor>,
}

impl Session {
    /// Creates a new `Session` instance for communicating with `TradingView`.
    ///
    /// This method generates a new session ID and sets up the necessary `WebSocket` Packet to create a new session
    /// and set the required fields for receiving price quotes. The resulting `Session` instance can be used to
    /// send and receive messages over the `WebSocket` connection.
    ///
    /// # Examples
    /// ```
    /// use trade_vision::quote::session::Session;
    ///
    /// let session = Session::new();
    /// ```
    ///
    pub async fn new() -> Self {
        let session_id = generate_session_id(None);
        let (tx_to_send, rx_to_send) = mpsc::channel::<String>(20);

        tx_to_send
            .send(
                WSPacket {
                    m: "quote_create_session",
                    p: into_inner_identifier(&session_id),
                }
                .format(),
            )
            .await
            .unwrap();

        tx_to_send
            .send(
                WSPacket {
                    m: "quote_set_fields",
                    p: [
                        vec![(session_id).clone()],
                        get_quote_fields(&FieldTypes::Price),
                    ]
                    .concat()
                    .into_ws_vec_values(),
                }
                .format(),
            )
            .await
            .unwrap();

        Self {
            session_id,
            tx_to_send,
            data: HashMap::new(),
            rx_to_send: Some(rx_to_send),
            processors: vec![convert_to_message_processor!(process_heartbeat)],
        }
    }

    pub async fn connect(&mut self) {
        // Connect to the WebSocket API and split the stream into read and write halves
        let mut request = CONNECTION.into_client_request().unwrap();
        request.headers_mut().append(
            http::header::ORIGIN,
            "https://s.tradingview.com".parse().unwrap(),
        );

        let (ws_stream, _) = connect_async(request).await.expect("Failed to connect");

        let (write, read) = ws_stream.split();

        // self.read = Some(read);

        let rx_to_send = self.rx_to_send.take().expect("rx_to_send is None");

        // Spawn a task to send messages to the server
        tokio::spawn(send_message(rx_to_send, write));
        tokio::spawn(handle_messages(
            read,
            self.tx_to_send.clone(),
            self.processors.clone(),
        ));

        // Send a message to the server to set the authorization token
        self.tx_to_send
            .send(
                WSPacket {
                    m: "set_auth_token",
                    p: into_inner_identifier("unauthorized_user_token"),
                }
                .format(),
            )
            .await
            .unwrap();
    }

    /// This is adds a symbol which data is retrieved for.
    ///
    /// It uses the api to request a symbol, then over
    /// the time interval data is sent to the client
    /// this data shows the price.
    pub async fn add_symbol(&self, to_add: &str) {
        if !self.data.keys().any(|i| i == to_add) {
            self.tx_to_send
                .send(
                    WSPacket {
                        m: "quote_add_symbols",
                        p: vec![&self.session_id.clone(), to_add].into_ws_vec_values(),
                    }
                    .format(),
                )
                .await
                .unwrap();
        }
    }

    /// Gets the price data for a given symbol.
    ///
    /// If the symbol exists in the data map, its internal data is modified to include the new price data.
    /// If the symbol does not exist in the data map, a new entry with the symbol and the new price data is added.
    #[must_use]
    pub fn get_data(&self, symbol: &str) -> (f64, f64) {
        self.data.get(symbol).map_or((0.0, 0.0), |internal_data| {
            internal_data.to_owned().to_owned()
        })
    }

    /// Sets the technical analysis (TA) data for a given symbol.
    ///
    /// If the symbol exists in the data map, its internal data is modified to include the new TA data.
    /// If the symbol does not exist in the data map, a new entry with the symbol and the new TA data is added.
    pub fn set_data_price(&mut self, symbol: &str, data: f64) {
        self.data
            .entry(symbol.to_owned())
            .and_modify(|x| *x = (data, x.1))
            .or_insert((data, 0.0));
    }

    /// Sets the technical analysis (TA) data for a symbol.
    ///
    /// Updates the internal data hashmap for the specified symbol with the TA data.
    /// If the symbol is not present in the hashmap, a new entry is created with TA data 0.0 for the price.
    pub fn set_data_ta(&mut self, symbol: &str, data: f64) {
        self.data
            .entry(symbol.to_owned())
            .and_modify(|x| *x = (x.0, data))
            .or_insert((0.0, data));
    }

    /// Returns a list of all symbols for which data has been retrieved.
    ///
    /// The returned list contains only the symbol names, without any associated data.
    #[must_use]
    pub fn keys(&self) -> hash_map::IntoKeys<std::string::String, (f64, f64)> {
        self.data.clone().into_keys()
    }

    // /// Process the incoming websocket stream
    // pub fn process_stream(&mut self) {
    //     let read = self.read.take().unwrap();
    //     let tx_to_send = self.tx_to_send.clone();
    //     let processors = self.processors.clone();
    //     // Create a new stream from the websocket
    //     tokio::spawn(handle_messages(read, tx_to_send, processors));

    //     // thread::spawn(move || {
    //     //     for a in 0..10 {
    //     //         println!("{a}");
    //     //     }
    //     // });
    // }

    pub fn add_processor(&mut self, processor: MessageProcessor) {
        self.processors.push(processor);
    }

    pub async fn process_messages(&self, data: String, tx_to_send: Sender<String>) {
        let parsed_data = parse_ws_packet(data); // Access data using Arc

        for d in parsed_data {
            for processor in &self.processors {
                let d = d.clone();
                let tx_to_send = tx_to_send.clone();
                let processor = *processor;

                tokio::spawn(async move {
                    let boxed_processor = processor(&d, tx_to_send);
                    boxed_processor.await;
                });
            }
        }
    }
}

async fn handle_messages(
    read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    tx_to_send: Sender<String>,
    processors: Processors,
) {
    // For each message received on the stream
    let reading = read.for_each(
        move |message: Result<Message, tokio_tungstenite::tungstenite::Error>| {
            // Clone the sender
            let tx_to_send = tx_to_send.clone();
            let processors = processors.clone();
            async move {
                if let Ok(message) = message {
                    if let Ok(text) = message.into_text() {
                        // Use `text` as a regular string or convert to &str if needed

                        println!("\x1b[91m🠳\x1b[0m {text}");

                        process_messages(&processors, text.to_string(), &tx_to_send);
                    }
                }
            }
        },
    );

    reading.await;
}

// `7MM"""Mq.
//   MM   `MM.
//   MM   ,M9 `7Mb,od8 ,pW"Wq.   ,p6"bo   .gP"Ya  ,pP"Ybd ,pP"Ybd  ,pW"Wq.`7Mb,od8 ,pP"Ybd
//   MMmmdM9    MM' "'6W'   `Wb 6M'  OO  ,M'   Yb 8I   `" 8I   `" 6W'   `Wb MM' "' 8I   `"
//   MM         MM    8M     M8 8M       8M"""""" `YMMMa. `YMMMa. 8M     M8 MM     `YMMMa.
//   MM         MM    YA.   ,A9 YM.    , YM.    , L.   I8 L.   I8 YA.   ,A9 MM     L.   I8
// .JMML.     .JMML.   `Ybmd9'   YMbmd'   `Mbmmd' M9mmmP' M9mmmP'  `Ybmd9'.JMML.   M9mmmP'

type Processors = Vec<MessageProcessor>;

fn process_messages(processors: &Processors, data: String, tx_to_send: &Sender<String>) {
    let processors = processors.clone();
    let parsed_data = parse_ws_packet(data);
    for d in parsed_data {
        for processor in &processors {
            tokio::spawn({
                let d: Packet<'_> = d.clone();
                let tx_to_send = tx_to_send.clone();
                let processor = *processor;
                async move {
                    processor(&d, tx_to_send).await;
                }
            });
        }
    }
}

// Thanks to help of rust forum: https://users.rust-lang.org/t/general-async-function-pointer/97997
// More thanks to the forum to help me fix lifetimes: https://users.rust-lang.org/t/guidance-on-custom-lifetimes-and-lifetime-function-parameters/99585/2
/// Type of function that can process messages, cannot be async
pub type MessageProcessor = for<'a> fn(&'a Packet<'a>, mpsc::Sender<String>) -> BoxFuture<'a, ()>;
// pub type MessageProcessorFunction = fn(&Packet, mpsc::Sender<String>) -> ();

// pub fn convert_to_message_processor<Fut: Future<Output = ()> + Send + 'static>(
//     f: impl Fn(String, mpsc::Sender<String>) -> Fut + 'static,
// ) -> MessageProcessor {
//     Box::new(move |message, tx_to_send| Box::pin(f(message, tx_to_send)))
// }

/// This is a type of function that is able to process a message from the `TradingView` websocket.
///
/// The function cannot be async because it is used in a for loop in the `process_stream` method and rust doesn't easily support async
/// function types
pub async fn process_heartbeat<'a>(message: &Packet<'a>, tx_to_send: mpsc::Sender<String>) {
    if let Packet::Ping(num) = message {
        let ping = format_ws_ping(num);
        tx_to_send.send(ping).await.unwrap();
    }
}

async fn send_message(
    mut rx: mpsc::Receiver<String>,
    mut interface: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
) {
    loop {
        match rx.recv().await {
            Some(data) => {
                println!("\x1b[92m🠱\x1b[0m {}", &data);

                let message = Message::from(data);

                interface.send(message).await.unwrap();
            }
            None => {
                // println!("continued");
                continue;
            }
        }
    }
}

///
/// There are two different types of fields that can be retrieved
/// either all the fields available or just the fields
/// that relate to price.
fn get_quote_fields(field: &FieldTypes) -> Vec<String> {
    match field {
        FieldTypes::All => FIELDS.map(std::borrow::ToOwned::to_owned).to_vec(),
        FieldTypes::Price => vec![
            "lp".to_owned(),
            "high_price".to_owned(),
            "low_price".to_owned(),
            "price_52_week_high".to_owned(),
            "price_52_week_low".to_owned(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_quote_fields() {
        let quote_price = get_quote_fields(&FieldTypes::Price);
        assert_eq!(
            quote_price,
            vec![
                "lp",
                "high_price",
                "low_price",
                "price_52_week_high",
                "price_52_week_low"
            ],
            "The quote fields should include only 5 fields"
        );

        let quote_all = get_quote_fields(&FieldTypes::All);
        assert_eq!(
            quote_all,
            FIELDS.to_vec(),
            "The quote fields should include all the fields"
        );
    }

    #[test]
    fn test_field_types() {
        // Test the `All` variant
        assert_eq!(
            FieldTypes::All,
            FieldTypes::All,
            "The `All` variant should be equal to itself"
        );
        assert_ne!(
            FieldTypes::All,
            FieldTypes::Price,
            "The `All` variant should not be equal to the `Price` variant"
        );

        // Test the `Price` variant
        assert_eq!(
            FieldTypes::Price,
            FieldTypes::Price,
            "The `Price` variant should be equal to itself"
        );
        assert_ne!(
            FieldTypes::Price,
            FieldTypes::All,
            "The `Price` variant should not be equal to the `All` variant"
        );
    }
}
