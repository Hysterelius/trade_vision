use crate::session::{ChartSession, Session};
use crate::utils::generate_session_id;
use crate::Error;

enum ChartTypes {
    HeikinAshi,
    Renko,
    LineBreak,
    Kagi,
    PointAndFigure,
    Range,
}

impl ChartTypes {
    fn to_string(&self) -> &str {
        match self {
            ChartTypes::HeikinAshi => "BarSetHeikenAshi@tv-basicstudies-60!",
            ChartTypes::Renko => "BarSetRenko@tv-prostudies-40!",
            ChartTypes::LineBreak => "BarSetPriceBreak@tv-prostudies-34!",
            ChartTypes::Kagi => "BarSetKagi@tv-prostudies-34!",
            ChartTypes::PointAndFigure => "BarSetPnF@tv-prostudies-34!",
            ChartTypes::Range => "BarSetRange@tv-basicstudies-72!",
        }
    }
}

// pub struct ChartSession {
//     ChartSessionID: String,
//     ReplaySessionID: String,
//     session: Session,
// }

impl Session {
    pub fn chart(&mut self) -> Result<(), Error> {
        match self.chart_details {
            Some(_) => return Err(Error::ChartSessionAlreadyInitialised()),
            None => {
                self.chart_details = Some(ChartSession {
                    chart_session_id: generate_session_id(Some("cs")),
                    replay_session_id: generate_session_id(Some("rs")),
                    replay_mode: false,
                });
            }
        }

        Ok(())
    }

    // self.sessions.insert(
    //     self.chartSessionID.clone(),
    //     Session {
    //         chartSessionID: self.chartSessionID.clone(),
    //         studyListeners: HashMap::new(),
    //         infos: HashMap::new(),
    //         chartSession: self,
    //         indexes: HashMap::new(),
    //         periods: HashMap::new(),
    //     },
    // );
}
