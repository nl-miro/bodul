//! MVP retail-sourcing pipeline (Phase 0).
//!
//! Library crate exposing the pipeline stages; the binaries in `src/bin/` drive
//! them.

/// Extension trait for mapping `Result<T, E>` into `Result<T, CommandError>`.
///
/// Avoids the repetitive `.map_err(|e| CommandError::Storage(e.to_string()))` closure
/// in every command handler.
pub trait IntoCommandError<T> {
    fn storage_err(self) -> Result<T, kernel::io::CommandError>;
    fn handler_err(self) -> Result<T, kernel::io::CommandError>;
}

impl<T, E: std::fmt::Display> IntoCommandError<T> for Result<T, E> {
    fn storage_err(self) -> Result<T, kernel::io::CommandError> {
        self.map_err(|e| kernel::io::CommandError::Storage(e.to_string()))
    }

    fn handler_err(self) -> Result<T, kernel::io::CommandError> {
        self.map_err(|e| kernel::io::CommandError::HandlerExecution(e.to_string()))
    }
}

// Stage A — retailer data (hardcoded in Phase 0) and sourcing trigger
pub mod retailer_sourcing;

// Application assembly and Mulac wiring
pub mod assembly;

// File-based logging setup shared by the binaries
pub mod logging;

// Stage B/C — sitemap fetching and processing
pub mod sitemap_discovery;

// Stage D — offer page fetching
pub mod offer_discovery;

// Stage E — offer page processing (typed models now live in
// `retailer-sourcing`'s per-retailer `offer_details::v1` modules)
pub mod offer_processing;

// Stage E — retailer-specific HTML parsing
pub mod html_parser;

// Stage G — product inventory (write side)
pub mod product_information_management;

// Stage H — offer inventory (write side)
pub mod offer_information_management;

// Stage K — customer-facing API and web app
pub mod customer_facing;

// Supporting module for database access
pub mod database;

// Supporting module for fetching stuff
pub mod retailer_data_ingestion;

// Diesel table declarations for generated and hand-written database code.
pub mod lib_sitemap;
pub mod schema;
//

//

//
//

//

//

#[derive(Debug, thiserror::Error)]
pub enum RecordMappingError {
    #[error("invalid retailer code in sitemap retrieval record: {0}")]
    InvalidRetailerCode(#[from] bodul_shared::retailer::RetailerCodeConversionError),

    #[error("invalid retrieval status in sitemap retrieval record: {0}")]
    InvalidStatus(#[from] crate::sitemap_discovery::io::RetrievalStatusConversionError),
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("connection pool error: {0}")]
    Connection(#[from] diesel::r2d2::PoolError),

    #[error("database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("unknown retailer code: {0}")]
    UnknownRetailerCode(String),

    #[error("invalid stored record: {0}")]
    InvalidRecord(#[from] RecordMappingError),

    #[error("{0}")]
    Unexpected(String),
}
