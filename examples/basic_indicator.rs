use trade_vision::session::constructor;

extern crate trade_vision;

#[tokio::main]
async fn main() {
    let mut session = constructor().await;

    session.connect().await;

    // Adds the ETH/USDT symbol to the session
    session.add_symbol("BINANCE:ETHUSDT").await;

    session.process_stream().await;
}
