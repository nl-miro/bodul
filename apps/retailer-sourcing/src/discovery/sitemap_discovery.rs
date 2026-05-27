use crate::app_event::RetailerSourcingEvent;
use crate::discovery::fetch::{FetchCategoryPage, FetchProductDetails};
use crate::discovery::{COMMAND_TYPE_PROCESS_SITEMAP, Policy, PolicySubscriber};
use crate::source_retailer::io::SitemapRetrieved;
use kernel::io::{CommandError, CommandHandlerPort, DbPool};
use kernel::{ApplicationCommand, ApplicationEvent};
use retailer_guild::io::RetailerCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const COMMAND_TYPE_PRODUCT_ENUMERATION: &str = "ProductEnumeration";
pub const COMMAND_TYPE_CATEGORY_ENUMERATION: &str = "CategoryEnumeration";
pub const EVENT_TYPE_SITEMAP_PROCESSED: &str = "SitemapProcessed";
pub const EVENT_TYPE_PRODUCTS_ENUMERATED: &str = "ProductsEnumerated";
pub const EVENT_TYPE_CATEGORIES_ENUMERATED: &str = "CategoriesEnumerated";

// ---- ProcessSitemap (entry: a downloaded sitemap file) --------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSitemap {
    pub retailer_code: RetailerCode,
    pub artifact_id: Uuid,
    pub url: String,
}

impl ProcessSitemap {
    /// Intention: a retrieved sitemap means one `ProcessSitemap` per sitemap file.
    pub fn from_event(event: &SitemapRetrieved) -> Vec<Self> {
        event
            .files
            .iter()
            .map(|file| ProcessSitemap {
                retailer_code: event.retailer_code.clone(),
                artifact_id: file.artifact_id,
                url: file.url.clone(),
            })
            .collect()
    }
}

impl ApplicationCommand for ProcessSitemap {
    fn command_type(&self) -> &'static str {
        COMMAND_TYPE_PROCESS_SITEMAP
    }
}

#[allow(dead_code)]
pub struct ProcessSitemapHandler {
    pool: DbPool,
}

impl ProcessSitemapHandler {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl CommandHandlerPort<ProcessSitemap, RetailerSourcingEvent> for ProcessSitemapHandler {
    fn execute(&self, _cmd: ProcessSitemap) -> Result<Vec<RetailerSourcingEvent>, CommandError> {
        // TODO: load the sitemap file and send it for product + category
        // enumeration (see docs/sourcing-flow.md). Should emit SitemapProcessed.
        Ok(Vec::new())
    }
}

// ---- ProductEnumeration ---------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductEnumeration {
    pub retailer_code: RetailerCode,
    pub artifact_id: Uuid,
}

impl ApplicationCommand for ProductEnumeration {
    fn command_type(&self) -> &'static str {
        COMMAND_TYPE_PRODUCT_ENUMERATION
    }
}

#[allow(dead_code)]
pub struct ProductEnumerationHandler {
    pool: DbPool,
}

impl ProductEnumerationHandler {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl CommandHandlerPort<ProductEnumeration, RetailerSourcingEvent> for ProductEnumerationHandler {
    fn execute(
        &self,
        _cmd: ProductEnumeration,
    ) -> Result<Vec<RetailerSourcingEvent>, CommandError> {
        // TODO: enumerate product URLs from the sitemap. Should emit ProductsEnumerated.
        Ok(Vec::new())
    }
}

// ---- CategoryEnumeration --------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryEnumeration {
    pub retailer_code: RetailerCode,
    pub artifact_id: Uuid,
}

impl ApplicationCommand for CategoryEnumeration {
    fn command_type(&self) -> &'static str {
        COMMAND_TYPE_CATEGORY_ENUMERATION
    }
}

#[allow(dead_code)]
pub struct CategoryEnumerationHandler {
    pool: DbPool,
}

impl CategoryEnumerationHandler {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl CommandHandlerPort<CategoryEnumeration, RetailerSourcingEvent> for CategoryEnumerationHandler {
    fn execute(
        &self,
        _cmd: CategoryEnumeration,
    ) -> Result<Vec<RetailerSourcingEvent>, CommandError> {
        // TODO: enumerate category URLs from the sitemap. Should emit CategoriesEnumerated.
        Ok(Vec::new())
    }
}

// ---- events ---------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapProcessed {
    pub retailer_code: RetailerCode,
    pub artifact_id: Uuid,
}

impl ApplicationEvent for SitemapProcessed {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_SITEMAP_PROCESSED
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductsEnumerated {
    pub retailer_code: RetailerCode,
    pub product_urls: Vec<String>,
}

impl ApplicationEvent for ProductsEnumerated {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_PRODUCTS_ENUMERATED
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoriesEnumerated {
    pub retailer_code: RetailerCode,
    pub category_urls: Vec<String>,
}

impl ApplicationEvent for CategoriesEnumerated {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_CATEGORIES_ENUMERATED
    }
}

// ---- policies -------------------------------------------------------------

/// A processed sitemap fans out to product enumeration.
pub struct ProductEnumerationPolicy;

impl Policy for ProductEnumerationPolicy {
    type Command = ProductEnumeration;
    const EVENT_NAME: &'static str = "SitemapProcessed";
    const SOURCE: &'static str = "event:SitemapProcessed";

    fn commands(_event: &RetailerSourcingEvent) -> Vec<ProductEnumeration> {
        // TODO: build a ProductEnumeration for the processed sitemap
        Vec::new()
    }
}

pub type ProductEnumerationSubscriber = PolicySubscriber<ProductEnumerationPolicy>;

/// A processed sitemap fans out to category enumeration.
pub struct CategoryEnumerationPolicy;

impl Policy for CategoryEnumerationPolicy {
    type Command = CategoryEnumeration;
    const EVENT_NAME: &'static str = "SitemapProcessed";
    const SOURCE: &'static str = "event:SitemapProcessed";

    fn commands(_event: &RetailerSourcingEvent) -> Vec<CategoryEnumeration> {
        // TODO: build a CategoryEnumeration for the processed sitemap
        Vec::new()
    }
}

pub type CategoryEnumerationSubscriber = PolicySubscriber<CategoryEnumerationPolicy>;

/// Enumerated products fan out to a product-details fetch per URL.
pub struct FetchProductDetailsPolicy;

impl Policy for FetchProductDetailsPolicy {
    type Command = FetchProductDetails;
    const EVENT_NAME: &'static str = "ProductsEnumerated";
    const SOURCE: &'static str = "event:ProductsEnumerated";

    fn commands(_event: &RetailerSourcingEvent) -> Vec<FetchProductDetails> {
        // TODO: one FetchProductDetails per enumerated product URL
        Vec::new()
    }
}

pub type FetchProductDetailsSubscriber = PolicySubscriber<FetchProductDetailsPolicy>;

/// Enumerated categories fan out to a category-page fetch (first page) per URL.
pub struct FetchCategoryPagePolicy;

impl Policy for FetchCategoryPagePolicy {
    type Command = FetchCategoryPage;
    const EVENT_NAME: &'static str = "CategoriesEnumerated";
    const SOURCE: &'static str = "event:CategoriesEnumerated";

    fn commands(_event: &RetailerSourcingEvent) -> Vec<FetchCategoryPage> {
        // TODO: one FetchCategoryPage (page 1) per enumerated category URL
        Vec::new()
    }
}

pub type FetchCategoryPageSubscriber = PolicySubscriber<FetchCategoryPagePolicy>;
