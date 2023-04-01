//! Manages the current TradingView session
//! allows for the receiving of data and the defining of protocols
use std::collections::hash_map;
use std::collections::HashMap;

use super::super::protocol::*;
use super::super::utils::*;
use tokio::sync::mpsc;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
enum FieldTypes {
    All,
    Price,
}

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

    pub fn get_data(&self, symbol: &str) -> (f64, f64) {
        match self.data.get(symbol) {
            Some(internal_data) => internal_data.to_owned().to_owned(),
            None => (0.0, 0.0),
        }
    }

    pub fn set_data_price(&mut self, symbol: &str, data: f64) {
        self.data
            .entry(symbol.to_owned())
            .and_modify(|x| *x = (data, (*x).1))
            .or_insert((data, 0.0));
    }

    pub fn set_data_ta(&mut self, symbol: &str, data: f64) {
        self.data
            .entry(symbol.to_owned())
            .and_modify(|x| *x = ((*x).0, data))
            .or_insert((0.0, data));
    }

    pub fn keys(&self) -> hash_map::IntoKeys<std::string::String, (f64, f64)> {
        self.data.clone().into_keys()
    }
}

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
