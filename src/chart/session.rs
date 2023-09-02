use crate::protocol::{into_inner_string, WSPacket};
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
    pub async fn new(session: Session) -> Self {
        let chart_session_id = generate_session_id(Some("cs"));
        // Not using send(), as this the initial function, which I don't want to be async as it has to be certain that the chart has been initialised
        session
            .tx_to_send
            .send(
                WSPacket {
                    m: "chart_create_session".to_string(),
                    p: into_inner_string(chart_session_id.clone()),
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

    pub async fn close(&mut self) -> Session {
        let session = self.session.as_ref().expect("No session to close");
        let _ = session
            .tx_to_send
            .send(
                WSPacket {
                    m: "chart_delete_session".to_string(),
                    p: into_inner_string(self.chart_session_id.clone()),
                }
                .format(),
            )
            .await;
        self.session
            .take()
            .map_or_else(|| panic!("No session to close"), |s| s)
    }
}

// pub fn process_chart_data(message: String, _tx_to_send: mpsc::Sender<String>) {
//     if message.contains("~h~") {
//         todo!();
//     }
// }
