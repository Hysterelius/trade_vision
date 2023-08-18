use tokio::signal;
use trade_vision::{chart::session::ChartSession, session::Session};

extern crate trade_vision;

#[tokio::main]
async fn main() {
    let mut session = Session::new().await;

    session.connect().await;
    println!("yes!");

    // Adds the ETH/USDT symbol to the session
    session.add_symbol("BINANCE:ETHUSDT").await;

    let _chart = ChartSession::new(session).await;

    signal::ctrl_c().await.unwrap();
}
