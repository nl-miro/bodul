use crate::daily_sourcing;
use crate::discovery::fetch::{
    self, FetchCategoryPageHandler, FetchProductDetailsHandler, NextCategoryPageSubscriber,
};
use crate::discovery::sitemap_discovery::{
    self, CategoryEnumerationHandler, CategoryEnumerationSubscriber, FetchCategoryPageSubscriber,
    FetchProductDetailsSubscriber, ProcessSitemapHandler, ProductEnumerationHandler,
    ProductEnumerationSubscriber,
};
use crate::discovery::website_discovery::{
    self, FetchNodeHandler, FetchNodeSubscriber, NodeFetchCategorySubscriber, ProcessMenuHandler,
};
use crate::discovery::{
    COMMAND_TYPE_PROCESS_MENU, COMMAND_TYPE_PROCESS_SITEMAP, RetailerHomepageRetrievedSubscriber,
    SitemapRetrievedSubscriber,
};
use crate::source_retailer;
use kernel::io::DbPool;
use kernel::{EventSubscriberPort, KernelConfig, KernelError, PersistentKernelHandle, boot};
use std::sync::Arc;

pub fn start_mulac(pool: DbPool) -> Result<PersistentKernelHandle, KernelError> {
    boot(KernelConfig::default())
        // ---- command handlers ---------------------------------------------
        .command_handler(
            daily_sourcing::io::COMMAND_TYPE,
            Arc::new(daily_sourcing::io::DailySourcingHandler::new()),
        )
        .command_handler(
            source_retailer::COMMAND_TYPE,
            Arc::new(source_retailer::io::SourceRetailerHandler::new(
                pool.clone(),
            )),
        )
        .command_handler(
            COMMAND_TYPE_PROCESS_SITEMAP,
            Arc::new(ProcessSitemapHandler::new(pool.clone())),
        )
        .command_handler(
            sitemap_discovery::COMMAND_TYPE_PRODUCT_ENUMERATION,
            Arc::new(ProductEnumerationHandler::new(pool.clone())),
        )
        .command_handler(
            sitemap_discovery::COMMAND_TYPE_CATEGORY_ENUMERATION,
            Arc::new(CategoryEnumerationHandler::new(pool.clone())),
        )
        .command_handler(
            fetch::COMMAND_TYPE_FETCH_PRODUCT_DETAILS,
            Arc::new(FetchProductDetailsHandler::new(pool.clone())),
        )
        .command_handler(
            fetch::COMMAND_TYPE_FETCH_CATEGORY_PAGE,
            Arc::new(FetchCategoryPageHandler::new(pool.clone())),
        )
        .command_handler(
            COMMAND_TYPE_PROCESS_MENU,
            Arc::new(ProcessMenuHandler::new(pool.clone())),
        )
        .command_handler(
            website_discovery::COMMAND_TYPE_FETCH_NODE,
            Arc::new(FetchNodeHandler::new(pool.clone())),
        )
        // ---- event subscribers (policies) ----------------------------------
        .event_subscriber_with_command_gateway(
            daily_sourcing::io::EVENT_TYPE_DAILY_SOURCING_REQUESTED,
            "daily-sourcing-fan-out",
            |command_gateway| {
                Arc::new(source_retailer::io::DailySourcingFanOutSubscriber::new(
                    command_gateway,
                )) as Arc<dyn EventSubscriberPort>
            },
        )
        .event_subscriber_with_command_gateway(
            source_retailer::EVENT_TYPE_SITEMAP_RETRIEVED,
            "sitemap-retrieved-fan-out",
            |command_gateway| {
                Arc::new(SitemapRetrievedSubscriber::new(command_gateway))
                    as Arc<dyn EventSubscriberPort>
            },
        )
        .event_subscriber_with_command_gateway(
            source_retailer::EVENT_TYPE_RETAILER_HOMEPAGE_RETRIEVED,
            "retailer-homepage-retrieved-fan-out",
            |command_gateway| {
                Arc::new(RetailerHomepageRetrievedSubscriber::new(command_gateway))
                    as Arc<dyn EventSubscriberPort>
            },
        )
        .event_subscriber_with_command_gateway(
            sitemap_discovery::EVENT_TYPE_SITEMAP_PROCESSED,
            "sitemap-processed-to-product-enumeration",
            |command_gateway| {
                Arc::new(ProductEnumerationSubscriber::new(command_gateway))
                    as Arc<dyn EventSubscriberPort>
            },
        )
        .event_subscriber_with_command_gateway(
            sitemap_discovery::EVENT_TYPE_SITEMAP_PROCESSED,
            "sitemap-processed-to-category-enumeration",
            |command_gateway| {
                Arc::new(CategoryEnumerationSubscriber::new(command_gateway))
                    as Arc<dyn EventSubscriberPort>
            },
        )
        .event_subscriber_with_command_gateway(
            sitemap_discovery::EVENT_TYPE_PRODUCTS_ENUMERATED,
            "products-enumerated-to-fetch-product-details",
            |command_gateway| {
                Arc::new(FetchProductDetailsSubscriber::new(command_gateway))
                    as Arc<dyn EventSubscriberPort>
            },
        )
        .event_subscriber_with_command_gateway(
            sitemap_discovery::EVENT_TYPE_CATEGORIES_ENUMERATED,
            "categories-enumerated-to-fetch-category-page",
            |command_gateway| {
                Arc::new(FetchCategoryPageSubscriber::new(command_gateway))
                    as Arc<dyn EventSubscriberPort>
            },
        )
        .event_subscriber_with_command_gateway(
            fetch::EVENT_TYPE_CATEGORY_PAGE_EXTRACTED,
            "category-page-extracted-to-next-page",
            |command_gateway| {
                Arc::new(NextCategoryPageSubscriber::new(command_gateway))
                    as Arc<dyn EventSubscriberPort>
            },
        )
        .event_subscriber_with_command_gateway(
            website_discovery::EVENT_TYPE_MENU_PROCESSED,
            "menu-processed-to-fetch-node",
            |command_gateway| {
                Arc::new(FetchNodeSubscriber::new(command_gateway)) as Arc<dyn EventSubscriberPort>
            },
        )
        .event_subscriber_with_command_gateway(
            website_discovery::EVENT_TYPE_NODE_FETCHED,
            "node-fetched-to-fetch-category-page",
            |command_gateway| {
                Arc::new(NodeFetchCategorySubscriber::new(command_gateway))
                    as Arc<dyn EventSubscriberPort>
            },
        )
        .start_persistent(pool, 0)
}
