//! Houses function for a collection of important TradingView functions
//! which do not fit into any other category.

use serde::{Deserialize, Serialize};

/// Returns a string indicating which stock exchange the input belongs to.
///
/// # Arguments
///
/// * `exchange` - A string slice containing the name of the exchange.
///
/// # Examples
///
/// ```
/// use trade_vision::misc_requests::get_screener;
/// assert_eq!(get_screener("Nyse"), "america");
/// assert_eq!(get_screener("Foo"), "foo");
///
/// ```
/// # Notes
///
/// The function converts the input `exchange` to uppercase before matching.
///
/// If the exchange is not matched it will just return the string provided but just in lowercase.
///
pub fn get_screener(exchange: &str) -> String {
    let uex = exchange.to_ascii_uppercase();
    let uexs = uex.as_str();

    match uexs {
        "NASDAQ" | "NYSE" | "NYSE ARCA" | "OTC" => "america".to_string(), // 🇺🇸 United States
        "ASX" => "australia".to_string(),                                 // 🇦🇺 Australia
        "TSX" | "TSXV" | "CSE" | "NEO" => "canada".to_string(),           // 🇨🇦 Canada
        "EGX" => "egypt".to_string(),                                     // 🇪🇬 Egypt
        "FWB" | "SWB" | "XETR" => "germany".to_string(),                  // 🇩🇪 Germany
        "BSE" | "NSE" => "india".to_string(),                             // 🇮🇳 India
        "TASE" => "israel".to_string(),                                   // 🇮🇱 Israel
        "MIL" | "MILSEDEX" => "italy".to_string(),                        // 🇮🇹 Italy
        "LUXSE" => "luxembourg".to_string(),                              // 🇱🇺 Luxembourg
        "NEWCONNECT" => "poland".to_string(),                             // 🇵🇱 Poland
        "NGM" => "sweden".to_string(),                                    // 🇸🇪 Sweden
        "BIST" => "turkey".to_string(),                                   // 🇹🇷 Turkey
        "LSE" | "LSIN" => "uk".to_string(),                               // 🇬🇧 United Kingdom
        "HNX" => "vietnam".to_string(),                                   // 🇻🇳 Vietnam
        "BINANCE" | "BITSTAMP" | "COINBASE" => "crypto".to_string(),      // 🅱️ Crypto
        _ => exchange.to_ascii_lowercase(),                               // 🏳️ Another exchange
    }
}

/// This struct contains the necessary data required to retrieve data
/// for a given symbol.
#[derive(Deserialize, Serialize, Debug)]
struct Symbol {
    symbols: Symbols,
    columns: Vec<String>,
}

// This struct is used to specify the tickers to get data for and the
/// types of data to retrieve.
#[derive(Deserialize, Serialize, Debug)]
struct Symbols {
    tickers: Vec<String>,
    query: Queries,
}

/// This struct is used to specify the types of data to retrieve from the TradingView server.
#[derive(Deserialize, Serialize, Debug)]
struct Queries {
    types: Vec<i32>,
}

/// This array contains the default indicator to retrieve data for.
pub const BASE_INDICATORS: [&str; 1] = ["Recommend.All"];

/// This function retrieves technical analysis data for the given symbols
/// using the provided interval and indicators.
///
/// # Arguments
///
/// * symbols - A vector of strings containing the symbols to retrieve data for.
/// * interval - A string containing the interval to retrieve data for.
/// * indicators - A vector of strings containing the indicators to retrieve data for.
///
/// # Returns
///
/// A f64 value containing the technical analysis data for the given symbols.
///
/// # Examples
///
/// ```
/// use trade_vision::misc_requests::get_ta;
///
/// async fn get_data() {
///     let symbol = "AAPL";
///     let indicators = vec!["Recommend.All"];
///     let interval = "1h";
///     let data = get_ta(vec![symbol], interval, indicators).await;
///     println!("Technical analysis for {}: {}", symbol, data);
/// }
/// ```
pub async fn get_ta(symbols: Vec<&str>, interval: &str, indicators: Vec<&str>) -> f64 {
    let client = reqwest::Client::new();

    let converted_interval = match interval {
        "1m" => "|1",
        "5m" => "|5",
        "15m" => "|15",
        "30m" => "|30",
        "1h" => "|60",
        "2h" => "|120",
        "4h" => "|240",
        "1w" => "|1W",
        "1M" => "|1M",
        _ => "",
    };

    let changed_indicators: Vec<String> = indicators
        .clone()
        .into_iter()
        .map(|x| String::from(x) + converted_interval)
        .collect();

    let json_data = Symbol {
        symbols: Symbols {
            tickers: symbols.iter().map(|x| x.to_string()).collect(),
            query: Queries { types: vec![] },
        },
        columns: changed_indicators,
    };

    let url = format!(
        "https://scanner.tradingview.com/{}/scan",
        get_screener((symbols[0].split(':').collect::<Vec<&str>>())[0])
    );

    let data: serde_json::Value = client
        .post(url)
        .json(&json_data)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // println!("{}", data["data"][0]["d"]);

    // let data = serde_json::to_value(12).expect("failed when value");

    data["data"][0]["d"][0].as_f64().unwrap_or(0.0)
}

#[test]
fn test_get_screener() {
    // 🇺🇸 United States
    assert_eq!(
        get_screener("NYSE"),
        "america",
        "Input 'NYSE' should return 'america'"
    );
    assert_eq!(
        get_screener("nyse"),
        "america",
        "Input 'nyse' should return 'america'"
    );
    assert_eq!(
        get_screener("NYSE ARCA"),
        "america",
        "Input 'NYSE ARCA' should return 'america'"
    );
    assert_eq!(
        get_screener("nyse arca"),
        "america",
        "Input 'nyse arca' should return 'america'"
    );
    assert_eq!(
        get_screener("NASDAQ"),
        "america",
        "Input 'NASDAQ' should return 'america'"
    );
    assert_eq!(
        get_screener("nasdaq"),
        "america",
        "Input 'nasdaq' should return 'america'"
    );
    assert_eq!(
        get_screener("OTC"),
        "america",
        "Input 'OTC' should return 'america'"
    );
    assert_eq!(
        get_screener("otc"),
        "america",
        "Input 'otc' should return 'america'"
    );

    // 🇦🇺 Australia
    assert_eq!(
        get_screener("ASX"),
        "australia",
        "Input 'ASX' should return 'australia'"
    );

    // 🇨🇦 Canada
    assert_eq!(
        get_screener("TSX"),
        "canada",
        "Input 'TSX' should return 'canada'"
    );
    assert_eq!(
        get_screener("TSXV"),
        "canada",
        "Input 'TSXV' should return 'canada'"
    );
    assert_eq!(
        get_screener("CSE"),
        "canada",
        "Input 'CSE' should return 'canada'"
    );
    assert_eq!(
        get_screener("NEO"),
        "canada",
        "Input 'NEO' should return 'canada'"
    );

    // 🇪🇬 Egypt
    assert_eq!(
        get_screener("EGX"),
        "egypt",
        "Input 'EGX' should return 'egypt'"
    );

    // 🇩🇪 Germany
    assert_eq!(
        get_screener("FWB"),
        "germany",
        "Input 'FWB' should return 'germany'"
    );
    assert_eq!(
        get_screener("SWB"),
        "germany",
        "Input 'SWB' should return 'germany'"
    );
    assert_eq!(
        get_screener("XETR"),
        "germany",
        "Input 'XETR' should return 'germany'"
    );

    // 🇮🇳 India
    assert_eq!(
        get_screener("BSE"),
        "india",
        "Input 'BSE' should return 'india'"
    );
    assert_eq!(
        get_screener("NSE"),
        "india",
        "Input 'NSE' should return 'india'"
    );

    // 🇮🇱 Israel
    assert_eq!(
        get_screener("TASE"),
        "israel",
        "Input 'TASE' should return 'israel'"
    );

    // 🇮🇹 Italy
    assert_eq!(
        get_screener("MIL"),
        "italy",
        "Input 'MIL' should return 'italy'"
    );
    assert_eq!(
        get_screener("MILSEDEX"),
        "italy",
        "Input 'MILSEDEX' should return 'italy'"
    );

    // 🇱🇺 Luxembourg
    assert_eq!(
        get_screener("LUXSE"),
        "luxembourg",
        "Input 'LUXSE' should return 'luxembourg'"
    );

    // 🇵🇱 Poland
    assert_eq!(
        get_screener("NEWCONNECT"),
        "poland",
        "Input 'NEWCONNECT' should return 'poland'"
    );

    // 🇸🇪 Sweden
    assert_eq!(
        get_screener("NGM"),
        "sweden",
        "Input 'NGM' should return 'sweden'"
    );

    // 🇹🇷 Turkey
    assert_eq!(
        get_screener("BIST"),
        "turkey",
        "Input 'BIST' should return 'turkey'"
    );

    // 🇬🇧 United Kingdom
    assert_eq!(get_screener("LSE"), "uk", "Input 'LSE' should return 'uk'");
    assert_eq!(
        get_screener("LSIN"),
        "uk",
        "Input 'LSIN' should return 'uk'"
    );

    // 🇻🇳 Vietnam
    assert_eq!(
        get_screener("HNX"),
        "vietnam",
        "Input 'HNX' should return 'vietnam'"
    );

    // 🏳️ Tests other exchange
    assert_eq!(
        get_screener("foo"),
        "foo",
        "Input 'foo' should return 'foo'"
    );
    assert_eq!(
        get_screener("FOO"),
        "foo",
        "Input 'FOO' should return 'foo'"
    );
}
