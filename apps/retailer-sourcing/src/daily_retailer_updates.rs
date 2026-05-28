/// Daily Retailer Updates feature
///
/// Handles the daily update of retailer data and emits events for downstream processing

pub mod io {
    use super::intention;

    pub use intention::{
        SitemapUpdateRequested, StartDailyRetailerUpdates, StartDailyRetailerUpdatesHandler,
        WebsiteUpdateRequested,
    };
}

mod intention {
    use crate::RetailerEvent;
    use crate::daily_retailer_updates::implementation::Err;
    use std::vec::IntoIter;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct StartDailyRetailerUpdates {
        pub retailer_id: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SitemapUpdateRequested {
        pub retailer_id: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct WebsiteUpdateRequested {
        pub retailer_id: String,
    }

    pub struct StartDailyRetailerUpdatesHandler;

    pub type BoxedEvent = Box<dyn RetailerEvent>;

    // Kicks off the daily update cycle: for every active retailer, emits one sitemap
    // and one website update event so the downstream discovery subscribers can run independently.
    impl StartDailyRetailerUpdatesHandler {
        pub fn execute(&self, _: StartDailyRetailerUpdates) -> Result<Vec<BoxedEvent>, Err> {
            let active_retailers = self.get_active_retailers();
            let events = self.request_sitemap_and_website_updates(active_retailers)?;

            Ok(events)
        }
    }
}

mod implementation {
    use crate::RetailerEvent;
    use crate::daily_retailer_updates::intention::{
        SitemapUpdateRequested, StartDailyRetailerUpdatesHandler, WebsiteUpdateRequested,
    };
    use std::fmt;
    use std::vec::IntoIter;

    impl RetailerEvent for SitemapUpdateRequested {
        fn event_type(&self) -> &'static str {
            Self::EVENT_TYPE
        }
    }

    impl RetailerEvent for WebsiteUpdateRequested {
        fn event_type(&self) -> &'static str {
            Self::EVENT_TYPE
        }
    }

    impl StartDailyRetailerUpdatesHandler {
        pub(crate) fn get_active_retailers(&self) -> IntoIter<String> {
            todo!()
        }
    }

    impl SitemapUpdateRequested {
        pub const EVENT_TYPE: &'static str = "retailer_sourcing.sitemap_update_requested";

        pub fn boxed(retailer_id: String) -> Box<dyn RetailerEvent> {
            let eve = SitemapUpdateRequested { retailer_id };

            Box::new(eve)
        }
    }

    impl WebsiteUpdateRequested {
        pub const EVENT_TYPE: &'static str = "retailer_sourcing.website_update_requested";

        pub fn boxed(retailer_id: String) -> Box<dyn RetailerEvent> {
            let eve = WebsiteUpdateRequested { retailer_id };

            Box::new(eve)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Err {
        ActiveRetailersUnavailable,
    }

    impl fmt::Display for Err {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Err::ActiveRetailersUnavailable => {
                    write!(f, "active retailers are unavailable")
                }
            }
        }
    }

    impl std::error::Error for Err {}

    impl StartDailyRetailerUpdatesHandler {
        pub fn request_sitemap_and_website_updates(
            &self,
            active_retailers: IntoIter<String>,
        ) -> Result<Vec<crate::daily_retailer_updates::intention::BoxedEvent>, Err> {
            let mut events: Vec<crate::daily_retailer_updates::intention::BoxedEvent> = vec![];

            events.extend(active_retailers.flat_map(|retailer| {
                [
                    SitemapUpdateRequested::boxed(retailer.clone()),
                    WebsiteUpdateRequested::boxed(retailer.clone()),
                ]
            }));

            Ok(events)
        }
    }
}

mod temp_helpers {
    use crate::daily_retailer_updates::intention::{
        SitemapUpdateRequested, StartDailyRetailerUpdates, WebsiteUpdateRequested,
    };
    use std::fmt;

    // TODO: Consider replacing with derive_more or custom display macro
    impl fmt::Display for StartDailyRetailerUpdates {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "StartDailyRetailerUpdates(retailer_id={})",
                self.retailer_id
            )
        }
    }

    // TODO: Consider replacing with derive_more or custom display macro
    impl fmt::Display for SitemapUpdateRequested {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "SitemapUpdateRequested(retailer_id={})",
                self.retailer_id
            )
        }
    }

    // TODO: Consider replacing with derive_more or custom display macro
    impl fmt::Display for WebsiteUpdateRequested {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "WebsiteUpdateRequested(retailer_id={})",
                self.retailer_id
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::daily_retailer_updates::intention::{
        SitemapUpdateRequested, StartDailyRetailerUpdates, WebsiteUpdateRequested,
    };

    #[test]
    fn test_update_retailers_daily_command() {
        let cmd = StartDailyRetailerUpdates {
            retailer_id: "retailer-123".to_string(),
        };
        assert_eq!(cmd.retailer_id, "retailer-123");
    }

    #[test]
    fn test_sitemap_update_event() {
        let event = SitemapUpdateRequested {
            retailer_id: "retailer-123".to_string(),
        };
        assert_eq!(
            event,
            SitemapUpdateRequested {
                retailer_id: "retailer-123".to_string()
            }
        );
    }

    #[test]
    fn test_website_update_event() {
        let event = WebsiteUpdateRequested {
            retailer_id: "retailer-123".to_string(),
        };
        assert_eq!(
            event,
            WebsiteUpdateRequested {
                retailer_id: "retailer-123".to_string()
            }
        );
    }

    #[test]
    fn test_update_event_display_uses_rust_struct_names() {
        let sitemap = SitemapUpdateRequested {
            retailer_id: "retailer-123".to_string(),
        };
        let website = WebsiteUpdateRequested {
            retailer_id: "retailer-123".to_string(),
        };

        assert_eq!(
            format!("{sitemap}"),
            "SitemapUpdateRequested(retailer_id=retailer-123)"
        );
        assert_eq!(
            format!("{website}"),
            "WebsiteUpdateRequested(retailer_id=retailer-123)"
        );
    }
}
