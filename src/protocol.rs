//! This is a module for formatting incoming and outgoing packets

use regex::Regex;
use serde::{Deserialize, Serialize};

/// A struct for representing a WebSocket packet.
///
/// # Fields
///
/// * `m` - A string representing the message.
/// * `p` - A vector of strings representing the parameters.
///
/// # Notes
/// A valid schema is `~m~X~m~{Y}~`
///
/// With `Y` being a valid json string
///
/// With `X` being the length of json string `Y`
///
#[derive(Serialize, Deserialize)]
pub struct WSPacket {
    pub m: String,
    pub p: Vec<String>,
}

impl WSPacket {
    pub fn format(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        format!("~m~{}~m~{}", json.len(), json)
    }
}

/// Formats a packet for sending to the server as a request in a valid TradingView schema
///
/// # Arguments
///
/// * `packet` - A [`WSPacket`] that is then formatted into compatible string
///
/// # Examples
///
/// ```
/// use trade_vision::protocol::{format_ws_packet, WSPacket};
/// let packet = WSPacket {
///     m: "foo".to_string(),
///     p: vec!["bar".to_string()],
/// };
///
/// let formatted_packet = format_ws_packet(packet);
///
/// assert_eq!(
///     formatted_packet, "~m~23~m~{\"m\":\"foo\",\"p\":[\"bar\"]}",
/// );
///
/// ```
/// # Notes
///
/// The strings are formatted into a json string for sending to the server.
///
/// The m value is the length of the resulting json string **it does not include the `\` to lint the `"`**
///
#[deprecated]
pub fn format_ws_packet(packet: WSPacket) -> String {
    let json = serde_json::to_string(&packet).unwrap();
    format!("~m~{}~m~{}", json.len(), json)
}

/// Formats a ping to keep the TradingView connection alive.
///
/// # Arguments
///
/// * `num` - The corresponding ping number
///
/// # Examples
///
/// ```
/// use trade_vision::protocol::format_ws_ping;
/// let formatted_ping = format_ws_ping(1);
///
/// assert_eq!(
///     formatted_ping,
///     "~m~4~m~~h~1",
/// );
///
/// ```
///
///
pub fn format_ws_ping(num: u32) -> String {
    format!("~m~{}~m~~h~{}", (num.to_string().len() + 3), num)
}

/// Takes a incoming TradingView packet and reformats it for interpretation.
///
/// # Arguments
///
/// * `packet` - The incoming packet in a form of a string
///
/// # Examples
///
/// ```
/// use trade_vision::protocol::parse_ws_packet;
/// let parsed_packet = parse_ws_packet("~m~4~m~~h~1");
///
/// assert_eq!(
///     parsed_packet,
///     vec!["~h~1"]
/// );
///
/// ```
///
///
pub fn parse_ws_packet(packet: &str) -> Vec<String> {
    // let cleaned_packet = packet.replace("~h~", "");
    let splitter_regex = Regex::new(r"~m~[0-9]{1,}~m~").unwrap();

    let packet_fields: Vec<String> = splitter_regex
        .split(packet)
        .filter(|x| !x.is_empty())
        .map(|x| x.to_owned())
        .collect();

    packet_fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_packet() {
        let packet = WSPacket {
            m: "foo".to_string(),
            p: vec!["bar".to_string()],
        };

        assert_eq!(
            packet.m, "foo",
            "The `m` field should have the String field 'hello'"
        );
        assert_eq!(
            packet.p,
            vec!["bar"],
            "The `p` field should the Vec field with ['world']"
        );
    }

    #[test]
    fn test_format_ws_packet() {
        let packet = WSPacket {
            m: "foo".to_string(),
            p: vec!["bar".to_string()],
        };

        let formatted_packet = packet.format();

        assert_eq!(
            formatted_packet, "~m~23~m~{\"m\":\"foo\",\"p\":[\"bar\"]}",
            "The packet should return a string of converted json with m of 'foo' and p of ['bar']"
        );
    }

    #[test]
    fn test_format_ws_ping() {
        let formatted_ping_length_one = format_ws_ping(1);
        assert_eq!(
            formatted_ping_length_one, "~m~4~m~~h~1",
            "The resulting ping should be 1, with length of 4 accounting for '~h~' and '1'"
        );

        let formatted_ping_length_two = format_ws_ping(22);
        assert_eq!(
            formatted_ping_length_two, "~m~5~m~~h~22",
            "The resulting ping should be 22, with length of 5 accounting for '~h~' and '22'"
        );

        let formatted_ping_length_three = format_ws_ping(333);
        assert_eq!(
            formatted_ping_length_three, "~m~6~m~~h~333",
            "The resulting ping should be 333, with length of 6 accounting for '~h~' and '333'"
        );
    }

    #[test]
    fn test_packet_parse() {
        let ping_parse = parse_ws_packet("~m~4~m~~h~1");

        assert_eq!(
            ping_parse,
            vec!["~h~1"],
            "The resulting ping should remove the length value and only return '~h~1'"
        );

        let packet_parse = parse_ws_packet(
            "~m~60~m~{\"m\":\"quote_completed\",\"p\":[\"xs_abcdABCD1234\",\"BITMEX:XBT\"]}",
        );

        assert_eq!(
            packet_parse,
            vec!["{\"m\":\"quote_completed\",\"p\":[\"xs_abcdABCD1234\",\"BITMEX:XBT\"]}"],
            "The resulting packet should remove the length value and account for all values"
        );

        let multi_packet_parse = parse_ws_packet("~m~626~m~{\"m\":\"qsd\",\"p\":[\"xs_abcdABCD1234\",{\"n\":\"BITMEX:XBT\",\"s\":\"ok\",\"v\":{\"volume\":1e+100,\"update_mode\":\"streaming\",\"typespecs\":[],\"type\":\"crypto\",\"short_name\":\"XBT\",\"pro_name\":\"BITMEX:XBT\",\"pricescale\":100,\"original_name\":\"BITMEX:XBT\",\"minmove2\":0,\"minmov\":1,\"lp_time\":1000000000,\"lp\":10000.11,\"listed_exchange\":\"BITMEX\",\"is_tradable\":true,\"fractional\":false,\"format\":\"price\",\"exchange\":\"BITMEX\",\"description\":\"Bitcoin / US Dollar Index\",\"current_session\":\"market\",\"currency_id\":\"USD\",\"currency_code\":\"USD\",\"currency-logoid\":\"country/US\",\"chp\":0.79,\"ch\":133.27,\"base_currency_id\":\"XTVCBTC\",\"base-currency-logoid\":\"crypto/XTVCBTC\"}}]}~m~60~m~{\"m\":\"quote_completed\",\"p\":[\"xs_abcdABCD1234\",\"BITMEX:XBT\"]}~m~60~m~{\"m\":\"quote_completed\",\"p\":[\"xs_abcdABCD1234\",\"BITMEX:XBT\"]}");

        assert_eq!(
            multi_packet_parse,
            vec!["{\"m\":\"qsd\",\"p\":[\"xs_abcdABCD1234\",{\"n\":\"BITMEX:XBT\",\"s\":\"ok\",\"v\":{\"volume\":1e+100,\"update_mode\":\"streaming\",\"typespecs\":[],\"type\":\"crypto\",\"short_name\":\"XBT\",\"pro_name\":\"BITMEX:XBT\",\"pricescale\":100,\"original_name\":\"BITMEX:XBT\",\"minmove2\":0,\"minmov\":1,\"lp_time\":1000000000,\"lp\":10000.11,\"listed_exchange\":\"BITMEX\",\"is_tradable\":true,\"fractional\":false,\"format\":\"price\",\"exchange\":\"BITMEX\",\"description\":\"Bitcoin / US Dollar Index\",\"current_session\":\"market\",\"currency_id\":\"USD\",\"currency_code\":\"USD\",\"currency-logoid\":\"country/US\",\"chp\":0.79,\"ch\":133.27,\"base_currency_id\":\"XTVCBTC\",\"base-currency-logoid\":\"crypto/XTVCBTC\"}}]}", "{\"m\":\"quote_completed\",\"p\":[\"xs_abcdABCD1234\",\"BITMEX:XBT\"]}", "{\"m\":\"quote_completed\",\"p\":[\"xs_abcdABCD1234\",\"BITMEX:XBT\"]}"],
            "The resulting packet should remove the length value and return 2 strings within a Vec"
        )
    }
}
