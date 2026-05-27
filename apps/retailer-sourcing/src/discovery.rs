pub mod extraction;
pub mod fetch;
pub mod sitemap_discovery;
pub mod website_discovery;

use crate::app_event::RetailerSourcingEvent;
use crate::discovery::sitemap_discovery::ProcessSitemap;
use crate::discovery::website_discovery::ProcessMenu;

use crate::discovery::policy::{Policy, PolicySubscriber};

pub const COMMAND_TYPE_PROCESS_SITEMAP: &str = "ProcessSitemap";
pub const COMMAND_TYPE_PROCESS_MENU: &str = "ProcessMenu";

pub mod io {
    pub use super::RetailerHomepageRetrievedSubscriber;
    pub use super::SitemapRetrievedSubscriber;
}

pub type SitemapRetrievedSubscriber = PolicySubscriber<SitemapRetrievedPolicy>;
pub type RetailerHomepageRetrievedSubscriber = PolicySubscriber<RetailerHomepageRetrievedPolicy>;

pub struct SitemapRetrievedPolicy;

impl Policy for SitemapRetrievedPolicy {
    type Command = ProcessSitemap;
    const EVENT_NAME: &'static str = "SitemapRetrieved";
    const SOURCE: &'static str = "event:SitemapRetrieved";

    fn commands(event: &RetailerSourcingEvent) -> Vec<ProcessSitemap> {
        match event {
            RetailerSourcingEvent::SitemapRetrieved(event) => ProcessSitemap::from_event(event),
            _ => Vec::new(),
        }
    }
}

//
//
//
pub struct RetailerHomepageRetrievedPolicy;

impl Policy for RetailerHomepageRetrievedPolicy {
    type Command = ProcessMenu;
    const EVENT_NAME: &'static str = "RetailerHomepageRetrieved";
    const SOURCE: &'static str = "event:RetailerHomepageRetrieved";

    fn commands(event: &RetailerSourcingEvent) -> Vec<ProcessMenu> {
        match event {
            RetailerSourcingEvent::RetailerHomepageRetrieved(event) => {
                vec![ProcessMenu::from_event(event)]
            }
            _ => Vec::new(),
        }
    }
}

//
//
//
//
//
//
//
//
//
//
//

pub mod policy {

    use kernel::io::{CommandGateway, NewCommand, NewCommandEnvelope};
    use std::marker::PhantomData;
    use std::sync::Arc;
    use uuid::Uuid;

    use crate::app_event::RetailerSourcingEvent;
    use kernel::{
        ApplicationCommand, EventError, EventSubscriberPort, NewCommandMetadata, NewEventEnvelope,
        NewEventMetadata,
    };

    /// Intention: "when event X happens, issue commands Y" (event-storming policy).
    /// `commands` returns empty for any non-matching variant, so each policy
    /// processes only the one event it cares about.
    pub trait Policy {
        type Command: ApplicationCommand;
        const EVENT_NAME: &'static str;
        const SOURCE: &'static str;

        fn commands(event: &RetailerSourcingEvent) -> Vec<Self::Command>;
    }

    /// Implementation: the generic subscriber shell shared by all policies.
    pub struct PolicySubscriber<P: Policy> {
        command_gateway: Arc<CommandGateway>,
        _policy: PhantomData<fn() -> P>,
    }

    impl<P: Policy> PolicySubscriber<P> {
        pub fn new(command_gateway: Arc<CommandGateway>) -> Self {
            Self {
                command_gateway,
                _policy: PhantomData,
            }
        }
    }

    impl<P: Policy> EventSubscriberPort for PolicySubscriber<P> {
        fn handle(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
            let meta = require_metadata(envelope, P::EVENT_NAME)?;

            for cmd in P::commands(&decode_event(envelope)?) {
                dispatch_command(
                    &self.command_gateway,
                    &cmd,
                    meta.correlation_id,
                    meta.event_id,
                    P::SOURCE,
                )?;
            }
            Ok(())
        }
    }

    fn require_metadata<'a>(
        envelope: &'a NewEventEnvelope,
        event_name: &str,
    ) -> Result<&'a NewEventMetadata, EventError> {
        let msg = format!("{event_name} event is missing metadata");

        envelope
            .metadata
            .as_ref()
            .ok_or_else(|| EventError::SubscriberExecution(msg))
    }

    fn decode_event(envelope: &NewEventEnvelope) -> Result<RetailerSourcingEvent, EventError> {
        serde_json::from_str(&envelope.payload)
            .map_err(|e| EventError::SubscriberExecution(e.to_string()))
    }

    fn dispatch_command<C: ApplicationCommand>(
        gateway: &CommandGateway,
        command: &C,
        correlation_id: Option<Uuid>,
        causation_event_id: Uuid,
        source: &str,
    ) -> Result<(), EventError> {
        let payload = serde_json::to_string(command)
            .map_err(|e| EventError::SubscriberExecution(e.to_string()))?;
        let envelope = NewCommandEnvelope {
            command: NewCommand {
                command_type: command.command_type().to_string(),
                payload,
            },
            metadata: Some(NewCommandMetadata {
                command_id: Uuid::now_v7(),
                correlation_id,
                causation_id: Some(causation_event_id),
                source: Some(source.to_string()),
            }),
        };
        gateway
            .dispatch(envelope)
            .map_err(|e| EventError::SubscriberExecution(e.to_string()))
    }
}
