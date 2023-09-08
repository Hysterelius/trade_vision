use tokio::signal;
use trade_vision::quote::session::Session;

extern crate trade_vision;

#[tokio::main]
async fn main() {
    let mut session = Session::new().await;

    session.connect().await;
    println!("yes!");

    // Adds the ETH/USDT symbol to the session
    session.add_symbol("BINANCE:ETHUSDT").await;

    // let _chart = Chart::new(session).await;

    signal::ctrl_c().await.unwrap();
}
