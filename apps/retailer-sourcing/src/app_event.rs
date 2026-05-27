pub mod io {
    pub use super::RetailerSourcingEvent;
}

use crate::daily_sourcing::io::DailySourcingRequested;
use crate::discovery::fetch::{CategoryPageExtracted, ProductDetailsExtracted};
use crate::discovery::sitemap_discovery::{
    CategoriesEnumerated, ProductsEnumerated, SitemapProcessed,
};
use crate::discovery::website_discovery::{MenuProcessed, NodeFetched};
use crate::source_retailer::io::{RetailerHomepageRetrieved, SitemapRetrieved};
use kernel::ApplicationEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum RetailerSourcingEvent {
    DailySourcingRequested(DailySourcingRequested),
    SitemapRetrieved(SitemapRetrieved),
    RetailerHomepageRetrieved(RetailerHomepageRetrieved),
    SitemapProcessed(SitemapProcessed),
    ProductsEnumerated(ProductsEnumerated),
    CategoriesEnumerated(CategoriesEnumerated),
    ProductDetailsExtracted(ProductDetailsExtracted),
    CategoryPageExtracted(CategoryPageExtracted),
    MenuProcessed(MenuProcessed),
    NodeFetched(NodeFetched),
}

impl ApplicationEvent for RetailerSourcingEvent {
    fn event_type(&self) -> &'static str {
        match self {
            Self::DailySourcingRequested(e) => e.event_type(),
            Self::SitemapRetrieved(e) => e.event_type(),
            Self::RetailerHomepageRetrieved(e) => e.event_type(),
            Self::SitemapProcessed(e) => e.event_type(),
            Self::ProductsEnumerated(e) => e.event_type(),
            Self::CategoriesEnumerated(e) => e.event_type(),
            Self::ProductDetailsExtracted(e) => e.event_type(),
            Self::CategoryPageExtracted(e) => e.event_type(),
            Self::MenuProcessed(e) => e.event_type(),
            Self::NodeFetched(e) => e.event_type(),
        }
    }
}
