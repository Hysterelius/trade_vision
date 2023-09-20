use std::ops::Deref;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WSPacket<'a> {
    pub m: &'a str,
    pub p: ArrayData<'a>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum WSVecValues<'a> {
    String(&'a str),
    InnerPriceData(Box<InnerPriceData<'a>>),
}

pub trait IntoWSVecValues<'a> {
    fn into_ws_vec_values(self) -> ArrayData<'a>;
}

impl<'a> IntoWSVecValues<'a> for Vec<&'a str> {
    fn into_ws_vec_values(self) -> ArrayData<'a> {
        ArrayData {
            identifier: self[0],
            data: Some(WSVecValues::String(self[1])),
        }
    }
}

impl<'a> IntoWSVecValues<'a> for &'a Vec<String> {
    fn into_ws_vec_values(self) -> ArrayData<'a> {
        ArrayData {
            identifier: &self[0],
            data: Some(WSVecValues::String(&self[1])),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct InnerPriceData<'a> {
    n: &'a str,
    s: &'a str,
    v: InnerPriceDataV,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct InnerPriceDataV {
    volume: Option<f64>,
    update_mode: Option<String>,
    typespecs: Option<Vec<String>>,
    r#type: Option<String>,
    short_name: Option<String>,
    pro_name: Option<String>,
    pricescale: Option<i32>,
    original_name: Option<String>,
    minmove2: Option<i32>,
    minmov: Option<i32>,
    lp_time: Option<i64>,
    lp: Option<f64>,
    listed_exchange: Option<String>,
    is_tradable: Option<bool>,
    fractional: Option<bool>,
    format: Option<String>,
    exchange: Option<String>,
    description: Option<String>,
    current_session: Option<String>,
    currency_id: Option<String>,
    currency_code: Option<String>,
    currency_logoid: Option<String>,
    chp: Option<f64>,
    ch: Option<f64>,
    base_currency_id: Option<String>,
    base_currency_logoid: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Packet<'a> {
    Ping(u32),
    WSPacket(Box<WSPacket<'a>>),
    Other(String),
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ArrayData<'a> {
    pub identifier: &'a str,
    pub data: Option<WSVecValues<'a>>,
}

#[must_use]
pub const fn into_inner_identifier(val: &str) -> ArrayData<'_> {
    ArrayData {
        identifier: val,
        data: None,
    }
}

impl<'a> WSPacket<'a> {
    #[must_use]
    pub fn format(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        format!("~m~{}~m~{}", json.len(), json)
    }
}

#[must_use]
pub fn format_ws_ping(num: &u32) -> String {
    format!("~m~{}~m~~h~{}", (num.to_string().len() + 3), num)
}

#[must_use]
pub fn parse_ws_packet<'a, S: AsRef<str> + 'a>(packet: S) -> Vec<Packet<'a>>
where
    std::string::String: std::convert::From<S>,
{
    let owned_string: String = packet.into();
    let leaked_str: &'static str = owned_string.leak();
    let packet_fields: Vec<&str> = split_on_msg_length(leaked_str);

    packet_fields
        .into_iter()
        .map(|p| {
            let packet = p; // Move the value of `packet` out of the closure.
            parse_each_packet(packet)
        }) // The value of `packet` is not borrowed by the closure.
        .collect::<Vec<Packet<'a>>>()
}

fn split_on_msg_length(packet: &str) -> Vec<&str> {
    let is_digits = |s: String| s.chars().all(|c| c.is_ascii_digit());

    packet
        .split("~m~")
        .filter(|x| !x.is_empty() && !is_digits((*x).to_string()))
        // .map(std::string::ToString::to_string)
        .collect()
}

#[must_use]
pub fn parse_each_packet(packet: &'static str) -> Packet<'static> {
    if packet.contains("~h~") {
        let num: u32 = packet
            .replace("~h~", "")
            .parse()
            .expect("Error turning ping into number");
        Packet::Ping(num)
    } else if packet.contains('m') {
        let ws_packet_result: Result<WSPacket<'static>, _> = serde_json::from_str(packet);

        Packet::WSPacket(Box::new(
            ws_packet_result.expect("Cannot turn packet into WSPacket using serde"),
        ))
    } else {
        Packet::Other(packet.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn test_ws_packet() {
        let packet = WSPacket {
            m: "foo",
            p: into_inner_identifier("bar"),
        };

        assert_eq!(
            packet.m, "foo",
            "The `m` field should have the String field 'hello'"
        );
        assert_eq!(
            packet.p,
            into_inner_identifier("bar"),
            "The `p` field should the Vec field with ['world']"
        );
    }

    #[test]
    fn test_format_ws_packet() {
        let packet = WSPacket {
            m: "foo",
            p: into_inner_identifier("bar"),
        };

        let formatted_packet = packet.format();

        assert_eq!(
            formatted_packet, "~m~23~m~{\"m\":\"foo\",\"p\":[\"bar\"]}",
            "The packet should return a string of converted json with m of 'foo' and p of ['bar']"
        );
    }

    #[test]
    fn test_format_ws_ping() {
        let formatted_ping_length_one = format_ws_ping(&1);
        assert_eq!(
            formatted_ping_length_one, "~m~4~m~~h~1",
            "The resulting ping should be 1, with length of 4 accounting for '~h~' and '1'"
        );

        let formatted_ping_length_two = format_ws_ping(&22);
        assert_eq!(
            formatted_ping_length_two, "~m~5~m~~h~22",
            "The resulting ping should be 22, with length of 5 accounting for '~h~' and '22'"
        );

        let formatted_ping_length_three = format_ws_ping(&333);
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
            vec![Packet::Ping(1)],
            "The resulting ping should remove the length value and only return '~h~1'"
        );

        let packet_parse = parse_ws_packet(
            "~m~60~m~{\"m\":\"quote_completed\",\"p\":[\"xs_abcdABCD1234\",\"BITMEX:XBT\"]}",
        );

        assert_eq!(
            packet_parse,
            vec![Packet::WSPacket(Box::new(WSPacket {
                m: "quote_completed",
                p: ArrayData {
                    identifier: "xs_abcdABCD1234",
                    data: Some(WSVecValues::String("BITMEX:XBT"))
                }
            }))],
            "The resulting packet should remove the length value and account for all values"
        );

        let multi_packet_parse = parse_ws_packet("~m~626~m~{\"m\":\"qsd\",\"p\":[\"xs_abcdABCD1234\",{\"n\":\"BITMEX:XBT\",\"s\":\"ok\",\"v\":{\"volume\":1e+100,\"update_mode\":\"streaming\",\"typespecs\":[],\"type\":\"crypto\",\"short_name\":\"XBT\",\"pro_name\":\"BITMEX:XBT\",\"pricescale\":100,\"original_name\":\"BITMEX:XBT\",\"minmove2\":0,\"minmov\":1,\"lp_time\":1000000000,\"lp\":10000.11,\"listed_exchange\":\"BITMEX\",\"is_tradable\":true,\"fractional\":false,\"format\":\"price\",\"exchange\":\"BITMEX\",\"description\":\"Bitcoin / US Dollar Index\",\"current_session\":\"market\",\"currency_id\":\"USD\",\"currency_code\":\"USD\",\"currency-logoid\":\"country/US\",\"chp\":0.79,\"ch\":133.27,\"base_currency_id\":\"XTVCBTC\",\"base-currency-logoid\":\"crypto/XTVCBTC\"}}]}~m~60~m~{\"m\":\"quote_completed\",\"p\":[\"xs_abcdABCD1234\",\"BITMEX:XBT\"]}~m~60~m~{\"m\":\"quote_completed\",\"p\":[\"xs_abcdABCD1234\",\"BITMEX:XBT\"]}");

        assert_eq!(
            multi_packet_parse,
            vec![
                Packet::WSPacket(Box::new(WSPacket {
                    m: "qsd",
                    p: ArrayData {
                        identifier: "xs_abcdABCD1234",
                        data: Some(WSVecValues::InnerPriceData(Box::new(InnerPriceData {
                            n: "BITMEX:XBT",
                            s: "ok",
                            v: InnerPriceDataV {
                                volume: Some(1e100),
                                update_mode: Some("streaming".to_string()),
                                typespecs: Some(vec![]),
                                r#type: Some("crypto".to_string()),
                                short_name: Some("XBT".to_string()),
                                pro_name: Some("BITMEX:XBT".to_string()),
                                pricescale: Some(100),
                                original_name: Some("BITMEX:XBT".to_string()),
                                minmove2: Some(0),
                                minmov: Some(1),
                                lp_time: Some(1_000_000_000),
                                lp: Some(10000.11),
                                listed_exchange: Some("BITMEX".to_string()),
                                is_tradable: Some(true),
                                fractional: Some(false),
                                format: Some("price".to_string()),
                                exchange: Some("BITMEX".to_string()),
                                description: Some("Bitcoin / US Dollar Index".to_string()),
                                current_session: Some("market".to_string()),
                                currency_id: Some("USD".to_string()),
                                currency_code: Some("USD".to_string()),
                                currency_logoid: None,
                                chp: Some(0.79),
                                ch: Some(133.27),
                                base_currency_id: Some("XTVCBTC".to_string()),
                                base_currency_logoid: None,
                            },
                        }))),
                    },
                })),
                Packet::WSPacket(Box::new(WSPacket {
                    m: "quote_completed",
                    p: ArrayData {
                        identifier: "xs_abcdABCD1234",
                        data: Some(WSVecValues::String("BITMEX:XBT"))
                    }
                })),
                Packet::WSPacket(Box::new(WSPacket {
                    m: "quote_completed",
                    p: ArrayData {
                        identifier: "xs_abcdABCD1234",
                        data: Some(WSVecValues::String("BITMEX:XBT"))
                    }
                }))
            ],
            "The resulting packet should remove the length value and return 2 strings within a Vec"
        );
    }

    #[test]
    fn test_single_packet_parse() {
        let packet_parse = parse_each_packet(
            "{\"m\":\"qsd\",\"p\":[\"xs_abcdABCD1234\",{\"n\":\"BITMEX:XBT\",\"s\":\"ok\",\"v\":{\"volume\":1e+100,\"update_mode\":\"streaming\",\"typespecs\":[],\"type\":\"crypto\",\"short_name\":\"XBT\",\"pro_name\":\"BITMEX:XBT\",\"pricescale\":100,\"original_name\":\"BITMEX:XBT\",\"minmove2\":0,\"minmov\":1,\"lp_time\":1000000000,\"lp\":10000.11,\"listed_exchange\":\"BITMEX\",\"is_tradable\":true,\"fractional\":false,\"format\":\"price\",\"exchange\":\"BITMEX\",\"description\":\"Bitcoin / US Dollar Index\",\"current_session\":\"market\",\"currency_id\":\"USD\",\"currency_code\":\"USD\",\"currency-logoid\":\"country/US\",\"chp\":0.79,\"ch\":133.27,\"base_currency_id\":\"XTVCBTC\",\"base-currency-logoid\":\"crypto/XTVCBTC\"}}]}",
        );

        assert_eq!(
            packet_parse,
            Packet::WSPacket(Box::new(WSPacket {
                m: "qsd",
                p: ArrayData {
                    identifier: "xs_abcdABCD1234",
                    data: Some(WSVecValues::InnerPriceData(Box::new(InnerPriceData {
                        n: "BITMEX:XBT",
                        s: "ok",
                        v: InnerPriceDataV {
                            volume: Some(1e100),
                            update_mode: Some("streaming".to_string()),
                            typespecs: Some(vec![]),
                            r#type: Some("crypto".to_string()),
                            short_name: Some("XBT".to_string()),
                            pro_name: Some("BITMEX:XBT".to_string()),
                            pricescale: Some(100),
                            original_name: Some("BITMEX:XBT".to_string()),
                            minmove2: Some(0),
                            minmov: Some(1),
                            lp_time: Some(1_000_000_000),
                            lp: Some(10000.11),
                            listed_exchange: Some("BITMEX".to_string()),
                            is_tradable: Some(true),
                            fractional: Some(false),
                            format: Some("price".to_string()),
                            exchange: Some("BITMEX".to_string()),
                            description: Some("Bitcoin / US Dollar Index".to_string()),
                            current_session: Some("market".to_string()),
                            currency_id: Some("USD".to_string()),
                            currency_code: Some("USD".to_string()),
                            currency_logoid: None,
                            chp: Some(0.79),
                            ch: Some(133.27),
                            base_currency_id: Some("XTVCBTC".to_string()),
                            base_currency_logoid: None,
                        },
                    }))),
                },
            })),
            "The resulting packet should remove the length value and account for all values"
        );
    }

    #[test]
    fn test_msg_split() {
        let message = "afjdkfja~m~123~m~fka";
        assert_eq!(split_on_msg_length(message), vec!["afjdkfja", "fka"]);
    }
}
