pub mod io {
    pub use super::RetailerSourcingEvent;
}

use crate::daily_sourcing::io::DailySourcingRequested;
use kernel::ApplicationEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum RetailerSourcingEvent {
    DailySourcingRequested(DailySourcingRequested),
}

impl ApplicationEvent for RetailerSourcingEvent {
    fn event_type(&self) -> &'static str {
        match self {
            Self::DailySourcingRequested(e) => e.event_type(),
        }
    }
}
