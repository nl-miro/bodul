const COMMAND_TYPE: &str = "SourceRetailer";

pub mod io {
    pub use super::application::{SourceRetailer, SourcingMethod};
    pub use super::eventing::DailySourcingFanOutSubscriber;
}

mod application {
    use chrono::{DateTime, Utc};
    use kernel::ApplicationCommand;
    use retailer_guild::io::RetailerCode;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub enum SourcingMethod {
        WebScraping,
        Sitemap,
    }

    impl SourcingMethod {
        pub fn all() -> &'static [SourcingMethod] {
            &[SourcingMethod::WebScraping, SourcingMethod::Sitemap]
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SourceRetailer {
        pub retailer_code: RetailerCode,
        pub method: SourcingMethod,
        pub requested_at: DateTime<Utc>,
    }

    impl ApplicationCommand for SourceRetailer {
        fn command_type(&self) -> &'static str {
            super::COMMAND_TYPE
        }
    }
}

mod eventing {
    use super::application::{SourceRetailer, SourcingMethod};
    use crate::app_event::io::RetailerSourcingEvent;
    use kernel::io::{CommandGateway, NewCommand, NewCommandEnvelope, NewCommandMetadata};
    use kernel::{EventError, EventSubscriberPort, NewEventEnvelope};
    use std::sync::Arc;
    use uuid::Uuid;

    pub struct DailySourcingFanOutSubscriber {
        command_gateway: Arc<CommandGateway>,
    }

    impl DailySourcingFanOutSubscriber {
        pub fn new(command_gateway: Arc<CommandGateway>) -> Self {
            Self { command_gateway }
        }
    }

    impl EventSubscriberPort for DailySourcingFanOutSubscriber {
        fn handle(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
            let meta = envelope.metadata.as_ref().ok_or_else(|| {
                EventError::SubscriberExecution(
                    "DailySourcingRequested event is missing metadata".to_string(),
                )
            })?;
            let wrapped: RetailerSourcingEvent = serde_json::from_str(&envelope.payload)
                .map_err(|e| EventError::SubscriberExecution(e.to_string()))?;
            let RetailerSourcingEvent::DailySourcingRequested(event) = wrapped;

            for retailer_code in &event.retailer_codes {
                for method in SourcingMethod::all() {
                    let cmd = SourceRetailer {
                        retailer_code: retailer_code.clone(),
                        method: *method,
                        requested_at: event.requested_at,
                    };
                    let payload = serde_json::to_string(&cmd)
                        .map_err(|e| EventError::SubscriberExecution(e.to_string()))?;
                    let gateway_envelope = NewCommandEnvelope {
                        command: NewCommand {
                            command_type: super::COMMAND_TYPE.to_string(),
                            payload,
                        },
                        metadata: Some(NewCommandMetadata {
                            command_id: Uuid::now_v7(),
                            correlation_id: meta.correlation_id,
                            causation_id: Some(meta.event_id),
                            source: Some("event:DailySourcingRequested".to_string()),
                        }),
                    };
                    self.command_gateway
                        .dispatch(gateway_envelope)
                        .map_err(|e| EventError::SubscriberExecution(e.to_string()))?;
                }
            }
            Ok(())
        }
    }
}
