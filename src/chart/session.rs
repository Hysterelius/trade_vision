use tokio::sync::mpsc;

use crate::protocol::{into_inner_identifier, Packet, WSPacket, WSVecValues};
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
pub struct Chart {
    session: Option<Session>,
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

impl Chart {
    /// .
    ///
    /// # Panics
    ///
    /// Panics if there is a fault creating the session.
    pub async fn new(session: Session) -> Self {
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

        Self {
            session: Some(session),
            chart_session_id,
            replay_session_id: generate_session_id(Some("rs")),
            replay_mode: false,
        }
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if that there is no session to close.
    pub async fn close(mut self) -> Session {
        let session: &Session = self.session.as_ref().expect("No session to close");
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

pub async fn process_chart_data(packet: &Packet<'_>, tx_to_send: mpsc::Sender<String>) {
    // if let Packets::Ping(num) = message {
    //     let ping = format_ws_ping(num);
    //     tx_to_send.send(ping).await.unwrap();
    // };

    if let Packet::WSPacket(packet) = packet {
        if let Some(WSVecValues::InnerPriceData(data)) = &packet.p.data {
            println!("{data:#?}");
        }
    }
}
