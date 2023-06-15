use trade_vision::{
    protocol::{format_ws_ping, parse_ws_packet},
    session::constructor,
};

use tokio_tungstenite::tungstenite::Message;

use futures_util::{pin_mut, StreamExt};

extern crate trade_vision;

#[tokio::main]
async fn main() {
    let mut session = constructor().await;

    session.connect().await;

    // Adds the ETH/USDT symbol to the session
    session.add_symbol("BINANCE:ETHUSDT").await;

    // Create a new stream from the websocket
    let ws_to_stream = {
        // For each message received
        session.read.take().expect("rx_to_send is None").for_each(
            |message: Result<Message, tokio_tungstenite::tungstenite::Error>| {
                // Clone the sender
                let tx_to_send = session.tx_to_send.clone();
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
                        if d.contains("~h~") {
                            let ping = format_ws_ping(d.replace("~h~", "").parse().unwrap());
                            tx_to_send.send(ping).await.unwrap();
                        }
                    }
                }
            },
        )
    };

    pin_mut!(ws_to_stream);
    ws_to_stream.await;
}
