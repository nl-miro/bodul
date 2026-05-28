// Retailer Sourcing service library

use std::fmt;

pub mod daily_retailer_updates;
pub mod sitemap_discovery;
pub mod website_discovery;

pub trait RetailerEvent: fmt::Display + fmt::Debug {
    /// Routing key stamped onto the envelope; the only thing a subscriber
    /// matches on, since event payloads can be structurally identical.
    fn event_type(&self) -> &'static str;
}
