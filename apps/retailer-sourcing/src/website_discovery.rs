/// Website Discovery feature
///
/// Subscribes to `website update requested` events, fetches and stores retailer homepages,
/// and processes the menu structure into a category tree.

pub mod io {
    pub use super::implementation::{CategoryNode, CategoryTree, HomepageContent};
    pub use super::intention::DiscoverWebsite;
}

mod intention {
    use kernel::{EventError, NewEventEnvelope as Envelope};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DiscoverWebsite {
        pub retailer_id: String,
        pub homepage_url: String,
    }

    pub struct WebsiteUpdateRequestedSubscriber {}

    impl WebsiteUpdateRequestedSubscriber {
        pub fn handle(&self, envelope: &Envelope) -> Result<Option<DiscoverWebsite>, EventError> {
            if self.not_supported(envelope) {
                return Ok(None);
            }

            let retailer_id = self.extract_retailer_id(envelope)?;

            let cmd = DiscoverWebsite::new(retailer_id, "TODO: add real url".to_string());

            Ok(Some(cmd))
        }
    }
}

mod implementation {
    use crate::daily_retailer_updates::io::WebsiteUpdateRequested;
    use crate::website_discovery::intention::{DiscoverWebsite, WebsiteUpdateRequestedSubscriber};
    use crate::website_discovery::temp_helpers::extract_payload;
    use kernel::{EventError, NewEventEnvelope};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct HomepageContent {
        pub retailer_id: String,
        pub url: String,
        pub html: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CategoryNode {
        pub name: String,
        pub url: String,
        pub children: Vec<CategoryNode>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CategoryTree {
        pub retailer_id: String,
        pub nodes: Vec<CategoryNode>,
    }

    impl WebsiteUpdateRequestedSubscriber {
        pub(crate) fn not_supported(&self, envelope: &NewEventEnvelope) -> bool {
            envelope.event_type != WebsiteUpdateRequested::EVENT_TYPE
        }

        pub(crate) fn extract_retailer_id(
            &self,
            envelope: &NewEventEnvelope,
        ) -> Result<String, EventError> {
            let event: WebsiteUpdateRequested = extract_payload(envelope)?;

            Ok(event.retailer_id)
        }
    }

    impl DiscoverWebsite {
        pub(crate) fn new(retailer_id: String, homepage_url: String) -> Self {
            Self {
                retailer_id,
                homepage_url,
            }
        }
    }
}

mod temp_helpers {
    use crate::daily_retailer_updates::io::WebsiteUpdateRequested;
    use crate::website_discovery::implementation::{CategoryNode, CategoryTree, HomepageContent};
    use crate::website_discovery::intention::DiscoverWebsite;
    use kernel::{EventError, EventMetadata, NewEventEnvelope};
    use std::fmt;

    // TODO: Consider replacing with derive_more or custom display macro
    impl fmt::Display for DiscoverWebsite {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "DiscoverWebsite(retailer_id={}, homepage_url={})",
                self.retailer_id, self.homepage_url
            )
        }
    }

    // TODO: Consider replacing with derive_more or custom display macro
    impl fmt::Display for HomepageContent {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "HomepageContent(retailer_id={}, url={})",
                self.retailer_id, self.url
            )
        }
    }

    // TODO: Consider replacing with derive_more or custom display macro
    impl fmt::Display for CategoryNode {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "CategoryNode(name={}, url={}, children={})",
                self.name,
                self.url,
                self.children.len()
            )
        }
    }

    // TODO: Consider replacing with derive_more or custom display macro
    impl fmt::Display for CategoryTree {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "CategoryTree(retailer_id={}, nodes={})",
                self.retailer_id,
                self.nodes.len()
            )
        }
    }

    pub fn dispatch_command(
        _command: &DiscoverWebsite,
        _correlation_id: &str,
        _causation_id: &str,
        _source: &str,
    ) -> Result<(), EventError> {
        todo!()
    }

    pub fn extract_payload(
        _envelope: &NewEventEnvelope,
    ) -> Result<WebsiteUpdateRequested, EventError> {
        todo!()
    }

    pub fn extract_metadata(_envelope: &NewEventEnvelope) -> Result<EventMetadata, EventError> {
        // TODO: Extract real event metadata from the incoming envelope once the transport shape exists.
        Ok(EventMetadata {
            event_id: uuid::Uuid::now_v7(),
            correlation_id: Some(uuid::Uuid::now_v7()),
            causation_id: Some(uuid::Uuid::now_v7()),
            source: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::website_discovery::implementation::{CategoryNode, CategoryTree, HomepageContent};
    use crate::website_discovery::intention::DiscoverWebsite;

    #[test]
    fn test_discover_website_command() {
        let cmd = DiscoverWebsite {
            retailer_id: "retailer-123".to_string(),
            homepage_url: "https://example.com".to_string(),
        };
        assert_eq!(cmd.retailer_id, "retailer-123");
        assert_eq!(cmd.homepage_url, "https://example.com");
    }

    #[test]
    fn test_homepage_content() {
        let content = HomepageContent {
            retailer_id: "retailer-123".to_string(),
            url: "https://example.com".to_string(),
            html: "<html></html>".to_string(),
        };
        assert_eq!(content.retailer_id, "retailer-123");
        assert_eq!(content.url, "https://example.com");
        assert_eq!(content.html, "<html></html>");
    }

    #[test]
    fn test_category_node() {
        let child = CategoryNode {
            name: "Sub Category".to_string(),
            url: "https://example.com/sub".to_string(),
            children: vec![],
        };
        let node = CategoryNode {
            name: "Top Category".to_string(),
            url: "https://example.com/top".to_string(),
            children: vec![child],
        };
        assert_eq!(node.name, "Top Category");
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].name, "Sub Category");
    }

    #[test]
    fn test_category_tree() {
        let tree = CategoryTree {
            retailer_id: "retailer-123".to_string(),
            nodes: vec![
                CategoryNode {
                    name: "Category A".to_string(),
                    url: "https://example.com/a".to_string(),
                    children: vec![],
                },
                CategoryNode {
                    name: "Category B".to_string(),
                    url: "https://example.com/b".to_string(),
                    children: vec![],
                },
            ],
        };
        assert_eq!(tree.retailer_id, "retailer-123");
        assert_eq!(tree.nodes.len(), 2);
    }
}
