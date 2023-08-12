use tokio::signal;
use trade_vision::session::constructor;

extern crate trade_vision;

#[tokio::main]
async fn main() {
    let mut session = constructor().await;

    session.connect().await;
    println!("yes!");

    // Adds the ETH/USDT symbol to the session
    session.add_symbol("BINANCE:ETHUSDT").await;

    // let mut chart = ChartSession::new(session);
    // println!("My code");
    // let mut session = chart.close().await;

    // session.process_stream();

    signal::ctrl_c().await.unwrap();

    // Wait for the stream task to complete
    // stream_task.await;
}
