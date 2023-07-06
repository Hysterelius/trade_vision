//! Manages the current TradingView session
//! allows for the receiving of data and the defining of protocols
use std::collections::hash_map;
use std::collections::HashMap;

use super::super::protocol::*;
use super::super::utils::*;
use futures_util::pin_mut;
use futures_util::stream::SplitStream;

use tokio::sync::mpsc;

use tokio_tungstenite::{
    connect_async, tungstenite::client::IntoClientRequest, tungstenite::Message, MaybeTlsStream,
    WebSocketStream,
};

use futures_util::{stream::SplitSink, SinkExt, StreamExt};

use tokio::net::TcpStream;

const CONNECTION: &str = "wss://data.tradingview.com/socket.io/websocket";

/// The two possible field types that can be used for data retrieval:
/// - All = all available TradingView fields/datapoints
/// - Price = only fields/datapoints related to price
#[allow(dead_code)]
#[derive(Debug, PartialEq)]
enum FieldTypes {
    All,
    Price,
}

/// The data related to a particular symbol
///
/// # Arguments
///
/// * `symbol`: The specified symbol that data is being collected for, in format `MARKET:SYMBOL` e.g., `NYSE:AAPL`
/// * `price`: A tokio mpsc sender stream, used for sending messages to the server
/// * `technical_analysis`: The current data from the datastream about prices and technical analysis, set by either 'set_data_price' or 'set_data_ta'
#[derive(Debug, Clone)]
pub struct SymbolData {
    pub symbol: String,
    pub price: f64,
    pub technical_analysis: f64,
}

/// All the possible fields for a TradingView session, impacts what is received
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

/// A session which encapsulates the current state of the TradingView session.
///
/// This session holds the id, the sending mpsc socket and the data that is incoming.
///
/// # Arguments
///
/// * `session_id`: The current id of the session, used to authenticate with TradingView, the session id
/// * `tx_to_send`: A tokio mpsc sender stream, used for sending messages to the server
/// * `data`: The current data from the datastream about prices and technical analysis, set by either 'set_data_price' or 'set_data_ta'
pub struct Session {
    session_id: String,
    pub tx_to_send: mpsc::Sender<String>,
    data: HashMap<String, (f64, f64)>,
    rx_to_send: Option<mpsc::Receiver<String>>,
    pub read: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    processors: Vec<MessageProcessor>,
}

impl Session {
    /// This initialises the TradingView session.
    ///
    /// It creates a unique ID for the session and sets the types of
    /// data received from the servers.
    pub async fn start(&self) {
        self.tx_to_send
            .send(format_ws_packet(WSPacket {
                m: "quote_create_session".to_string(),
                p: vec![(self.session_id).to_owned()],
            }))
            .await
            .unwrap();

        self.tx_to_send
            .send(format_ws_packet(WSPacket {
                m: "quote_set_fields".to_string(),
                p: [
                    vec![(self.session_id).to_owned()],
                    get_quote_fields(FieldTypes::Price),
                ]
                .concat(),
            }))
            .await
            .unwrap();
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

        self.read = Some(read);

        let rx_to_send = self.rx_to_send.take().expect("rx_to_send is None");

        // Spawn a task to send messages to the server
        tokio::spawn(send_message(rx_to_send, write));

        // Send a message to the server to set the authorization token
        self.tx_to_send
            .send(format_ws_packet(WSPacket {
                m: "set_auth_token".to_owned(),
                p: vec!["unauthorized_user_token".to_owned()],
            }))
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
                .send(format_ws_packet(WSPacket {
                    m: "quote_add_symbols".to_string(),
                    p: vec![self.session_id.to_owned(), to_add.to_owned()],
                }))
                .await
                .unwrap();
        }
    }

    /// Gets the price data for a given symbol.
    ///
    /// If the symbol exists in the data map, its internal data is modified to include the new price data.
    /// If the symbol does not exist in the data map, a new entry with the symbol and the new price data is added.
    pub fn get_data(&self, symbol: &str) -> (f64, f64) {
        match self.data.get(symbol) {
            Some(internal_data) => internal_data.to_owned().to_owned(),
            None => (0.0, 0.0),
        }
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
    pub fn keys(&self) -> hash_map::IntoKeys<std::string::String, (f64, f64)> {
        self.data.clone().into_keys()
    }

    /// Process the incoming websocket stream
    pub async fn process_stream(&mut self) {
        // Create a new stream from the websocket
        let ws_to_stream = {
            // For each message received
            self.read.take().expect("rx_to_send is None").for_each(
                |message: Result<Message, tokio_tungstenite::tungstenite::Error>| {
                    // Clone the sender
                    let tx_to_send = self.tx_to_send.clone();
                    let processors = self.processors.clone();
                    async move {
                        // Unwrap the message
                        let data = message
                            .expect("Message is an invalid format")
                            .into_text()
                            .expect("Could not turn into text");
                        // Parse the message
                        let parsed_data = parse_ws_packet(&data);
                        // Print the message to the terminal
                        println!("\x1b[91m🠳\x1b[0m {:#?}", data);

                        // For each parsed message
                        for d in parsed_data {
                            // If the message is a heartbeat, send a heartbeat back
                            for processor in processors.iter() {
                                {
                                    let d = d.clone();
                                    let tx_to_send = tx_to_send.clone();
                                    let processor = processor.clone();
                                    tokio::task::spawn_blocking(move || {
                                        processor(&d, tx_to_send.clone());
                                    })
                                    .await
                                    .expect("Task panicked")
                                }
                            }
                        }
                    }
                },
            )
        };

        pin_mut!(ws_to_stream);
        ws_to_stream.await;
    }

    pub fn add_processor(&mut self, processor: MessageProcessor) {
        self.processors.push(processor);
    }
}

async fn send_message(
    mut rx: mpsc::Receiver<String>,
    mut interface: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
) {
    loop {
        match rx.recv().await {
            Some(data) => {
                println!("\x1b[92m🠱\x1b[0m {:#?}", &data);

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

/// This asynchronous function is used to construct a new TradingView session. It takes in a
/// `mpsc::Sender<String>` as an argument which is used to send data from the session to the client.
///
/// It generates a unique ID for the session using the generate_session_id function, and uses it to
/// create a new Session instance. This instance is returned after invoking the start method
/// which initializes the session and sets the types of data received from the servers.
///
/// # Arguments
///
/// * tx_to_send - An instance of `mpsc::Sender<String>` that is used to send data from the session to the client.
///
/// # Examples
///
/// ```
/// use trade_vision::session;
///
/// #[tokio::main]
/// async fn main() {
///     let session = session::constructor().await;
/// }
/// ```
pub async fn constructor() -> Session {
    let session_id = generate_session_id(None);

    let (tx_to_send, rx_to_send) = mpsc::channel::<String>(20);

    let current_session = Session {
        session_id,
        tx_to_send,
        data: HashMap::new(),
        rx_to_send: Some(rx_to_send),
        read: None,
        processors: vec![process_heartbeat],
    };

    current_session.start().await;

    current_session
}
///
/// There are two different types of fields that can be retrieved
/// either all the fields available or just the fields
/// that relate to price.
fn get_quote_fields(field: FieldTypes) -> Vec<String> {
    match field {
        FieldTypes::All => FIELDS.map(|x| x.to_owned()).to_vec(),
        FieldTypes::Price => vec![
            "lp".to_owned(),
            "high_price".to_owned(),
            "low_price".to_owned(),
            "price_52_week_high".to_owned(),
            "price_52_week_low".to_owned(),
        ],
    }
}

pub type MessageProcessor = fn(&str, mpsc::Sender<String>);

/// This is a type of function that is able to process a message from the TradingView websocket.
/// The function cannot be async because it is used in a for loop in the `process_stream` method and rust doesn't easily support async
/// function types
// TODO: change to support async https://stackoverflow.com/questions/66769143/rust-passing-async-function-pointers https://users.rust-lang.org/t/how-to-store-async-function-pointer/38343/4
pub fn process_heartbeat<'a>(message: &'a str, tx_to_send: mpsc::Sender<String>) {
    if message.contains("~h~") {
        let ping = format_ws_ping(message.replace("~h~", "").parse().unwrap());
        tx_to_send.blocking_send(ping).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_quote_fields() {
        let quote_price = get_quote_fields(FieldTypes::Price);
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

        let quote_all = get_quote_fields(FieldTypes::All);
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
