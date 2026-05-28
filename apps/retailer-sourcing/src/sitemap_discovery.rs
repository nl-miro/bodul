/// Sitemap Discovery feature
///
/// Subscribes to `sitemap update requested` events, fetches and stores sitemap files,
/// and processes them for categories, products and misc data.

pub mod io {
    pub use super::implementation::{SitemapEntry, SitemapKind};
    pub use super::intention::DiscoverSitemap;
}

mod intention {
    use kernel::{EventError, NewEventEnvelope as Envelope};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DiscoverSitemap {
        pub retailer_id: String,
        pub sitemap_url: String,
    }

    pub struct SitemapUpdateRequestedSubscriber {}

    impl SitemapUpdateRequestedSubscriber {
        pub fn handle(&self, envelope: &Envelope) -> Result<Option<DiscoverSitemap>, EventError> {
            if self.not_supported(envelope) {
                return Ok(None);
            }

            let retailer_id = self.extract_retailer_id(envelope)?;

            let cmd = DiscoverSitemap::new(retailer_id, "TODO: add real url".to_string());

            Ok(Some(cmd))
        }
    }
}

mod implementation {
    use crate::daily_retailer_updates::io::SitemapUpdateRequested;
    use crate::sitemap_discovery::intention::{DiscoverSitemap, SitemapUpdateRequestedSubscriber};
    use crate::sitemap_discovery::temp_helpers::extract_payload;
    use kernel::{EventError, NewEventEnvelope};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum SitemapKind {
        Category,
        Product,
        Misc,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SitemapEntry {
        pub retailer_id: String,
        pub url: String,
        pub kind: SitemapKind,
    }

    impl SitemapUpdateRequestedSubscriber {
        pub(crate) fn not_supported(&self, envelope: &NewEventEnvelope) -> bool {
            envelope.event_type != SitemapUpdateRequested::EVENT_TYPE
        }

        pub(crate) fn extract_retailer_id(
            &self,
            envelope: &NewEventEnvelope,
        ) -> Result<String, EventError> {
            let event: SitemapUpdateRequested = extract_payload(envelope)?;

            Ok(event.retailer_id)
        }
    }

    impl DiscoverSitemap {
        pub(crate) fn new(retailer_id: String, sitemap_url: String) -> Self {
            Self {
                retailer_id,
                sitemap_url,
            }
        }
    }
}

mod temp_helpers {
    use crate::daily_retailer_updates::io::SitemapUpdateRequested;
    use crate::sitemap_discovery::implementation::{SitemapEntry, SitemapKind};
    use crate::sitemap_discovery::intention::DiscoverSitemap;
    use kernel::{EventError, EventMetadata, NewEventEnvelope};
    use std::fmt;
    use uuid::Uuid;

    // TODO: Consider replacing with derive_more or custom display macro
    impl fmt::Display for DiscoverSitemap {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "DiscoverSitemap(retailer_id={}, sitemap_url={})",
                self.retailer_id, self.sitemap_url
            )
        }
    }

    // TODO: Consider replacing with derive_more or custom display macro
    impl fmt::Display for SitemapKind {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                SitemapKind::Category => write!(f, "category"),
                SitemapKind::Product => write!(f, "product"),
                SitemapKind::Misc => write!(f, "misc"),
            }
        }
    }

    // TODO: Consider replacing with derive_more or custom display macro
    impl fmt::Display for SitemapEntry {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "SitemapEntry(retailer_id={}, url={}, kind={})",
                self.retailer_id, self.url, self.kind
            )
        }
    }

    pub fn dispatch_command(
        _command: &DiscoverSitemap,
        _correlation_id: &str,
        _causation_id: &str,
        _source: &str,
    ) -> Result<(), EventError> {
        todo!()
    }

    pub fn extract_payload(
        _envelope: &NewEventEnvelope,
    ) -> Result<SitemapUpdateRequested, EventError> {
        todo!()
    }

    pub fn extract_metadata(_envelope: &NewEventEnvelope) -> Result<EventMetadata, EventError> {
        // TODO: Extract real event metadata from the incoming envelope once the transport shape exists.
        Ok(EventMetadata {
            event_id: Uuid::now_v7(),
            correlation_id: Some(Uuid::now_v7()),
            causation_id: Some(Uuid::now_v7()),
            source: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::sitemap_discovery::implementation::{SitemapEntry, SitemapKind};
    use crate::sitemap_discovery::intention::DiscoverSitemap;

    #[test]
    fn test_discover_sitemap_command() {
        let cmd = DiscoverSitemap {
            retailer_id: "retailer-123".to_string(),
            sitemap_url: "https://example.com/sitemap.xml".to_string(),
        };
        assert_eq!(cmd.retailer_id, "retailer-123");
        assert_eq!(cmd.sitemap_url, "https://example.com/sitemap.xml");
    }

    #[test]
    fn test_sitemap_entry_category() {
        let entry = SitemapEntry {
            retailer_id: "retailer-123".to_string(),
            url: "https://example.com/categories".to_string(),
            kind: SitemapKind::Category,
        };
        assert_eq!(entry.kind, SitemapKind::Category);
    }

    #[test]
    fn test_sitemap_entry_product() {
        let entry = SitemapEntry {
            retailer_id: "retailer-123".to_string(),
            url: "https://example.com/products".to_string(),
            kind: SitemapKind::Product,
        };
        assert_eq!(entry.kind, SitemapKind::Product);
    }

    #[test]
    fn test_sitemap_entry_misc() {
        let entry = SitemapEntry {
            retailer_id: "retailer-123".to_string(),
            url: "https://example.com/other".to_string(),
            kind: SitemapKind::Misc,
        };
        assert_eq!(entry.kind, SitemapKind::Misc);
    }
}
