use tokio::sync::mpsc;

use crate::protocol::{
    format_ws_ping, into_inner_identifier, InnerPriceData, Packets, WSPacket, WSVecValues,
};
use crate::quote::session::Session;
use crate::utils::generate_session_id;

#[allow(unused)]
enum ChartTypes {
    HeikinAshi,
    Renko,
    LineBreak,
    Kagi,
    PointAndFigure,
    Range,
}

#[allow(unused)]
pub struct Chart<'a> {
    session: Option<Session<'a>>,
    chart_session_id: String,
    replay_session_id: String,
    replay_mode: bool,
}

#[allow(unused)]
impl ChartTypes {
    const fn to_string(&self) -> &str {
        match self {
            Self::HeikinAshi => "BarSetHeikenAshi@tv-basicstudies-60!",
            Self::Renko => "BarSetRenko@tv-prostudies-40!",
            Self::LineBreak => "BarSetPriceBreak@tv-prostudies-34!",
            Self::Kagi => "BarSetKagi@tv-prostudies-34!",
            Self::PointAndFigure => "BarSetPnF@tv-prostudies-34!",
            Self::Range => "BarSetRange@tv-basicstudies-72!",
        }
    }
}

impl<'a> Chart<'a> {
    pub async fn new(session: Session<'static>) -> Chart<'_> {
        let chart_session_id = generate_session_id(Some("cs"));
        // Not using send(), as this the initial function, which I don't want to be async as it has to be certain that the chart has been initialised
        session
            .tx_to_send
            .send(
                WSPacket {
                    m: "chart_create_session",
                    p: into_inner_identifier(&chart_session_id.clone()),
                }
                .format(),
            )
            .await
            .unwrap();

        Chart {
            session: Some(session),
            chart_session_id,
            replay_session_id: generate_session_id(Some("rs")),
            replay_mode: false,
        }
    }

    pub async fn close(&'static mut self) -> Session {
        let session: &Session<'static> = self.session.as_ref().expect("No session to close");
        let _ = session
            .tx_to_send
            .send(
                WSPacket {
                    m: "chart_delete_session",
                    p: into_inner_identifier(&self.chart_session_id.clone()),
                }
                .format(),
            )
            .await;
        self.session
            .take()
            .map_or_else(|| panic!("No session to close"), |s| s)
    }
}

pub async fn process_chart_data(message: &Packets<'_>, tx_to_send: mpsc::Sender<String>) {
    // if let Packets::Ping(num) = message {
    //     let ping = format_ws_ping(num);
    //     tx_to_send.send(ping).await.unwrap();
    // };

    if let Packets::WSPacket(packet) = message {
        if let WSVecValues::InnerPriceData(data) = &packet.p[0] {
            println!("{:?}", data);
        }
    }
}
