//! Manages the current TradingView session
//! allows for the receiving of data and the defining of protocols
use std::collections::hash_map;
use std::collections::HashMap;

use super::super::protocol::*;
use super::super::utils::*;
use tokio::sync::mpsc;

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
    tx_to_send: mpsc::Sender<String>,
    data: HashMap<String, (f64, f64)>,
}

impl Session {
    /// This initialises the TradingView session.
    ///
    /// It creates a unique ID for the session and sets the types of
    /// data received from the servers.
    pub async fn start(&self) {
        let _ = self
            .tx_to_send
            .send(format_ws_packet(WSPacket {
                m: "quote_create_session".to_string(),
                p: vec![(self.session_id).to_owned()],
            }))
            .await
            .unwrap();

        let _ = self
            .tx_to_send
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

    /// This is adds a symbol which data is retrieved for.
    ///
    /// It uses the api to request a symbol, then over
    /// the time interval data is sent to the client
    /// this data shows the price.
    pub async fn add_symbol(&self, to_add: &str) {
        if !self.data.keys().any(|i| i == to_add) {
            let _ = self
                .tx_to_send
                .send(format_ws_packet(WSPacket {
                    m: "quote_add_symbols".to_string(),
                    p: vec![(&self.session_id).to_owned(), to_add.to_owned()],
                }))
                .await
                .unwrap();
        }
    }

    /// Sets the price data for a given symbol.
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
            .and_modify(|x| *x = (data, (*x).1))
            .or_insert((data, 0.0));
    }

    /// Sets the technical analysis (TA) data for a symbol.
    ///
    /// Updates the internal data hashmap for the specified symbol with the TA data.
    /// If the symbol is not present in the hashmap, a new entry is created with TA data 0.0 for the price.
    pub fn set_data_ta(&mut self, symbol: &str, data: f64) {
        self.data
            .entry(symbol.to_owned())
            .and_modify(|x| *x = ((*x).0, data))
            .or_insert((0.0, data));
    }

    /// Returns a list of all symbols for which data has been retrieved.
    ///
    /// The returned list contains only the symbol names, without any associated data.
    pub fn keys(&self) -> hash_map::IntoKeys<std::string::String, (f64, f64)> {
        self.data.clone().into_keys()
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
/// use std::sync::mpsc;
/// use tradingview_websocket_api::{Session, constructor};
///
/// #[tokio::main]
/// async fn main() {
///     let (tx, _) = mpsc::channel();
///     let session = constructor(tx).await;
///     assert_eq!(session.get_session_id().len(), 36);
/// }
/// ```
pub async fn constructor(tx_to_send: mpsc::Sender<String>) -> Session {
    let session_id = generate_session_id(None);

    let current_session = Session {
        session_id: session_id,
        tx_to_send: tx_to_send,
        data: HashMap::new(),
    };

    current_session.start().await;

    current_session
}

/// This sets the fields to be retrieved from TradingView.
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
