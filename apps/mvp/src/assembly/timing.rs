//! Timing instrumentation for command/event processing.
//!
//! Two thin decorators wrap the kernel's handler/subscriber ports and record how
//! long each command/event took to process — its id, retailer code, and elapsed
//! time in ms — onto the dedicated timing log (see [`crate::logging::record_timing`]).
//! They delegate straight through, so the wrapped handler/subscriber is unchanged.
//!
//! The command handler port only receives the deserialized command (not the kernel
//! envelope), so the logged id is the command's own domain id and the retailer is
//! taken from the command payload where present — [`ProcessSitemap`] and
//! [`GroupSitemapContent`] don't carry one, so their retailer is left blank.
//! Event subscribers receive the full envelope, so the event id comes from the
//! envelope metadata and the retailer from the parsed payload.

use std::sync::Arc;
use std::time::Instant;

use kernel::io::{CommandError, CommandHandlerPort};
use kernel::{EventError, EventSubscriberPort, NewEventEnvelope};
use serde::de::DeserializeOwned;
use bodul_shared::retailer::RetailerCode;
use uuid::Uuid;

use crate::assembly::io::MvpEvent;
use crate::offer_discovery::io::DownloadOfferPage;
use crate::sitemap_discovery::io::{GroupSitemapContent, ProcessSitemap, RequestSitemapRetrieval};

/// A command whose processing time can be logged: exposes its domain id and, when
/// the payload carries it, its retailer code.
pub trait TimedCommand {
    fn timing_id(&self) -> Uuid;
    fn timing_retailer(&self) -> Option<RetailerCode> {
        None
    }
}

impl TimedCommand for RequestSitemapRetrieval {
    fn timing_id(&self) -> Uuid {
        self.retrieval_id
    }
    fn timing_retailer(&self) -> Option<RetailerCode> {
        Some(self.retailer_code)
    }
}

impl TimedCommand for ProcessSitemap {
    fn timing_id(&self) -> Uuid {
        self.retrieval_id
    }
}

impl TimedCommand for GroupSitemapContent {
    fn timing_id(&self) -> Uuid {
        self.processed_sitemap_id
    }
}

impl TimedCommand for DownloadOfferPage {
    fn timing_id(&self) -> Uuid {
        self.grouped_content_id
    }
    fn timing_retailer(&self) -> Option<RetailerCode> {
        Some(self.retailer_code)
    }
}

/// Wraps a command handler, timing each `execute` and logging id/retailer/elapsed.
struct TimedCommandHandler<C> {
    name: &'static str,
    inner: Arc<dyn CommandHandlerPort<C, MvpEvent>>,
}

impl<C: TimedCommand> CommandHandlerPort<C, MvpEvent> for TimedCommandHandler<C> {
    fn execute(&self, command: C) -> Result<Vec<MvpEvent>, CommandError> {
        let id = command.timing_id();
        let retailer = command.timing_retailer().map(|code| code.slug());

        let start = Instant::now();
        let result = self.inner.execute(command);
        let elapsed_ms = start.elapsed().as_millis() as u64;

        let status = if result.is_ok() { "ok" } else { "error" };
        crate::logging::record_timing(
            "command",
            self.name,
            &id.to_string(),
            retailer.as_deref(),
            elapsed_ms,
            status,
        );
        result
    }
}

/// Wraps `inner` so each command it processes is timed and logged under `name`.
pub fn timed_command<C>(
    name: &'static str,
    inner: Arc<dyn CommandHandlerPort<C, MvpEvent>>,
) -> Arc<dyn CommandHandlerPort<C, MvpEvent>>
where
    C: TimedCommand + DeserializeOwned + Send + Sync + 'static,
{
    Arc::new(TimedCommandHandler { name, inner })
}

/// Wraps an event subscriber, timing each `handle` and logging id/retailer/elapsed.
struct TimedEventSubscriber {
    name: &'static str,
    inner: Arc<dyn EventSubscriberPort>,
}

impl EventSubscriberPort for TimedEventSubscriber {
    fn handle(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
        let id = envelope.metadata.as_ref().map(|metadata| metadata.event_id);
        let retailer = retailer_of(&envelope.payload).map(|code| code.slug());

        let start = Instant::now();
        let result = self.inner.handle(envelope);
        let elapsed_ms = start.elapsed().as_millis() as u64;

        let status = if result.is_ok() { "ok" } else { "error" };
        let id = id.map(|id| id.to_string());
        crate::logging::record_timing(
            "event",
            self.name,
            id.as_deref().unwrap_or("-"),
            retailer.as_deref(),
            elapsed_ms,
            status,
        );
        result
    }
}

/// Best-effort retailer code from an event payload (the timed subscribers all carry
/// one); `None` if the payload doesn't parse.
fn retailer_of(payload: &str) -> Option<RetailerCode> {
    match serde_json::from_str::<MvpEvent>(payload).ok()? {
        MvpEvent::SitemapRetrieved(event) => Some(event.retailer_code),
        MvpEvent::SitemapProcessed(event) => Some(event.retailer_code),
        MvpEvent::SitemapContentGrouped(event) => Some(event.retailer_code),
        MvpEvent::OfferPageWasDownloaded(event) => Some(event.retailer_code),
        MvpEvent::OfferPageDownloadSkipped(event) => Some(event.retailer_code),
    }
}

/// Wraps `inner` so each event it processes is timed and logged under `name`.
pub fn timed_event(name: &'static str, inner: Arc<dyn EventSubscriberPort>) -> Arc<dyn EventSubscriberPort> {
    Arc::new(TimedEventSubscriber { name, inner })
}
