//! Shared "fetch a page → extract data" commands (the doc's `FetchAndExtract`,
//! split into two concrete commands). Skeleton only — no logic yet.

use crate::app_event::RetailerSourcingEvent;
use crate::discovery::{Policy, PolicySubscriber};
use kernel::io::{CommandError, CommandHandlerPort, DbPool};
use kernel::{ApplicationCommand, ApplicationEvent};
use retailer_guild::io::RetailerCode;
use serde::{Deserialize, Serialize};

pub const COMMAND_TYPE_FETCH_PRODUCT_DETAILS: &str = "FetchProductDetails";
pub const COMMAND_TYPE_FETCH_CATEGORY_PAGE: &str = "FetchCategoryPage";
pub const EVENT_TYPE_PRODUCT_DETAILS_EXTRACTED: &str = "ProductDetailsExtracted";
pub const EVENT_TYPE_CATEGORY_PAGE_EXTRACTED: &str = "CategoryPageExtracted";

// ---- commands -------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchProductDetails {
    pub retailer_code: RetailerCode,
    pub url: String,
}

impl ApplicationCommand for FetchProductDetails {
    fn command_type(&self) -> &'static str {
        COMMAND_TYPE_FETCH_PRODUCT_DETAILS
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchCategoryPage {
    pub retailer_code: RetailerCode,
    pub url: String,
    pub page: u32,
}

impl ApplicationCommand for FetchCategoryPage {
    fn command_type(&self) -> &'static str {
        COMMAND_TYPE_FETCH_CATEGORY_PAGE
    }
}

// ---- events ---------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductDetailsExtracted {
    pub retailer_code: RetailerCode,
    pub url: String,
}

impl ApplicationEvent for ProductDetailsExtracted {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_PRODUCT_DETAILS_EXTRACTED
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryPageExtracted {
    pub retailer_code: RetailerCode,
    pub url: String,
    pub next_page: Option<u32>,
}

impl ApplicationEvent for CategoryPageExtracted {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_CATEGORY_PAGE_EXTRACTED
    }
}

// ---- handlers -------------------------------------------------------------

#[allow(dead_code)]
pub struct FetchProductDetailsHandler {
    pool: DbPool,
}

impl FetchProductDetailsHandler {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl CommandHandlerPort<FetchProductDetails, RetailerSourcingEvent> for FetchProductDetailsHandler {
    fn execute(
        &self,
        _cmd: FetchProductDetails,
    ) -> Result<Vec<RetailerSourcingEvent>, CommandError> {
        // TODO: fetch the product details page and run ExtractProductDetailsData
        // (see docs/sourcing-flow.md). Should emit ProductDetailsExtracted.
        Ok(Vec::new())
    }
}

#[allow(dead_code)]
pub struct FetchCategoryPageHandler {
    pool: DbPool,
}

impl FetchCategoryPageHandler {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl CommandHandlerPort<FetchCategoryPage, RetailerSourcingEvent> for FetchCategoryPageHandler {
    fn execute(&self, _cmd: FetchCategoryPage) -> Result<Vec<RetailerSourcingEvent>, CommandError> {
        // TODO: fetch the category page, run SliceProducts → ExtractCategoryProductData
        // (see docs/sourcing-flow.md). Should emit CategoryPageExtracted with the
        // next page, if any.
        Ok(Vec::new())
    }
}

// ---- policies -------------------------------------------------------------

/// Paging: a fetched category page that has a next page triggers fetching it.
pub struct NextCategoryPagePolicy;

impl Policy for NextCategoryPagePolicy {
    type Command = FetchCategoryPage;
    const EVENT_NAME: &'static str = "CategoryPageExtracted";
    const SOURCE: &'static str = "event:CategoryPageExtracted";

    fn commands(_event: &RetailerSourcingEvent) -> Vec<FetchCategoryPage> {
        // TODO: if the extracted page reports a next page, fetch it
        Vec::new()
    }
}

pub type NextCategoryPageSubscriber = PolicySubscriber<NextCategoryPagePolicy>;
