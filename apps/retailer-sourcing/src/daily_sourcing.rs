pub mod io {
    pub use super::application::{
        COMMAND_TYPE, DailySourcingRequested, EVENT_TYPE_DAILY_SOURCING_REQUESTED,
        StartDailySourcing,
    };
    pub use super::handler::DailySourcingHandler;
    pub use super::http::DailySourcingResponse;
    pub use super::http::{DAILY_SOURCING_PATH, register, trigger};
}

mod domain {}

mod application {
    use chrono::{DateTime, Utc};
    use kernel::{ApplicationCommand, ApplicationEvent};
    use retailer_guild::io::RetailerCode;
    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    pub const COMMAND_TYPE: &str = "StartDailySourcing";
    pub const EVENT_TYPE_DAILY_SOURCING_REQUESTED: &str = "DailySourcingRequested";

    #[derive(Debug, Serialize, Deserialize)]
    pub struct StartDailySourcing {
        triggered_at: DateTime<Utc>,
    }

    impl ApplicationCommand for StartDailySourcing {
        fn command_type(&self) -> &'static str {
            COMMAND_TYPE
        }
    }

    impl StartDailySourcing {
        pub fn new(triggered_at: DateTime<Utc>) -> Self {
            Self { triggered_at }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DailySourcingRequested {
        pub retailer_codes: Vec<RetailerCode>,
        pub requested_at: DateTime<Utc>,
    }

    impl ApplicationEvent for DailySourcingRequested {
        fn event_type(&self) -> &'static str {
            EVENT_TYPE_DAILY_SOURCING_REQUESTED
        }
    }

    #[derive(Debug, Error)]
    pub enum DailySourcingError {
        #[error("Infrastructure error: {0}")]
        Infra(String),
    }
}

mod handler {
    use super::application::{DailySourcingRequested, StartDailySourcing};
    use crate::app_event::io::RetailerSourcingEvent;
    use crate::assembly::io::ActiveRetailers;
    use chrono::Utc;
    use kernel::io::{CommandError, CommandHandlerPort};

    pub struct DailySourcingHandler;

    impl DailySourcingHandler {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for DailySourcingHandler {
        fn default() -> Self {
            Self::new()
        }
    }

    impl CommandHandlerPort<StartDailySourcing, RetailerSourcingEvent> for DailySourcingHandler {
        fn execute(
            &self,
            _cmd: StartDailySourcing,
        ) -> Result<Vec<RetailerSourcingEvent>, CommandError> {
            let event = DailySourcingRequested {
                retailer_codes: ActiveRetailers::list(),
                requested_at: Utc::now(),
            };
            Ok(vec![RetailerSourcingEvent::DailySourcingRequested(event)])
        }
    }
}

mod http {
    use crate::assembly::io::BearerAuth;
    use crate::daily_sourcing::application::{DailySourcingError, StartDailySourcing};
    use chrono::Utc;
    use kernel::{
        NewCommandEnvelope,
        NewCommandMetadata,
        PersistentKernelState, //
    };
    use poem::error::ResponseError;
    use poem::web::Json;
    use poem::{
        EndpointExt,
        IntoResponse,
        Response,
        Result,
        Route,
        handler,
        http::StatusCode,
        middleware::AddData,
        post,
        web::Data, //
    };
    use serde::Serialize;
    use std::sync::Arc;
    use tokio::task::spawn_blocking;
    use uuid::Uuid;

    pub const DAILY_SOURCING_PATH: &str = "/daily-sourcing/";

    #[derive(Debug, Serialize)]
    pub struct DailySourcingResponse {
        pub command_id: Uuid,
    }

    impl ResponseError for DailySourcingError {
        fn status(&self) -> StatusCode {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    #[handler]
    pub async fn trigger(state: Data<&PersistentKernelState>) -> Result<Response> {
        let state = state.0.clone();
        let command_id = Uuid::now_v7();
        let envelope = NewCommandEnvelope {
            command: StartDailySourcing::new(Utc::now()),
            metadata: NewCommandMetadata {
                command_id,
                correlation_id: Some(command_id),
                causation_id: None,
                source: Some("retailer-sourcing.http".to_string()),
            },
        };

        spawn_blocking(move || state.dispatch_command(envelope))
            .await
            .map_err(|e| DailySourcingError::Infra(e.to_string()))?
            .map_err(|e| DailySourcingError::Infra(e.to_string()))?;

        Ok((
            StatusCode::ACCEPTED,
            Json(DailySourcingResponse { command_id }),
        )
            .into_response())
    }

    pub fn register(route: Route, state: Arc<PersistentKernelState>, bearer: Arc<String>) -> Route {
        route.at(
            DAILY_SOURCING_PATH,
            post(trigger)
                .with(AddData::new((*state).clone()))
                .with(BearerAuth::new((*bearer).clone())),
        )
    }
}
