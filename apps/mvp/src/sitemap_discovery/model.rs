use super::grouping::GroupedLinks;
use crate::lib_sitemap::io::SitemapDocument;
use crate::retailer_data_ingestion::FetchError;
use crate::sitemap_discovery::processing::io::SitemapParseError;
use bodul_shared::retailer::RetailerCode;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum RetrievalStatus {
    Requested,
    Retrieved,
    Failed,
    Processed,
    Grouped,
}

#[derive(Debug, thiserror::Error, PartialEq)]
#[error("unknown retrieval status: {0}")]
pub struct RetrievalStatusConversionError(String);

impl TryFrom<&str> for RetrievalStatus {
    type Error = RetrievalStatusConversionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "requested" => Ok(Self::Requested),
            "retrieved" => Ok(Self::Retrieved),
            "failed" => Ok(Self::Failed),
            "processed" => Ok(Self::Processed),
            "grouped" => Ok(Self::Grouped),
            unknown => Err(RetrievalStatusConversionError(unknown.to_string())),
        }
    }
}

impl TryFrom<String> for RetrievalStatus {
    type Error = RetrievalStatusConversionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl From<RetrievalStatus> for String {
    fn from(status: RetrievalStatus) -> Self {
        status.as_str().to_string()
    }
}

impl RetrievalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Retrieved => "retrieved",
            Self::Failed => "failed",
            Self::Processed => "processed",
            Self::Grouped => "grouped",
        }
    }
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum SitemapError {
    #[error("failed to fetch {url}: {source}")]
    Fetch {
        url: String,
        #[source]
        source: FetchError,
    },

    #[error("failed to parse {url}: {source}")]
    Parse {
        url: String,
        #[source]
        source: SitemapParseError,
    },

    #[error("raw sitemap document is missing for {url}")]
    MissingRawDocument { url: String },

    #[error("sitemap index contains a cycle at {url}")]
    CyclicReference { url: String },

    #[error("already fetched url {url}")]
    AlreadyFetched { url: String },

    #[error("sitemap nesting exceeds the maximum depth of {max_depth} at {url}")]
    MaximumDepthExceeded { url: String, max_depth: usize },

    #[error("sitemap configuration contains no root URLs")]
    NoRootSitemaps,

    #[error("sitemap contains {count} URLs, which exceeds the supported maximum")]
    TooManyUrls { count: usize },

    #[error("no sitemap configuration for retailer {retailer_code:?}")]
    NoSitemapConfig { retailer_code: RetailerCode },
}

impl SitemapError {
    pub fn parse(url: &str, source: SitemapParseError) -> SitemapError {
        SitemapError::Parse {
            url: url.to_string(),
            source,
        }
    }

    pub fn missing_raw_document(url: &str) -> SitemapError {
        SitemapError::MissingRawDocument { url: url.to_string() }
    }

    pub fn maximum_depth_exceeded(url: String, max_depth: usize) -> SitemapError {
        SitemapError::MaximumDepthExceeded { url, max_depth }
    }

    pub fn cyclic_reference(url: &str) -> SitemapError {
        SitemapError::CyclicReference { url: url.to_string() }
    }

    pub fn no_sitemap_config(retailer_code: RetailerCode) -> Self {
        SitemapError::NoSitemapConfig { retailer_code }
    }
}

#[cfg(test)]
mod retrieval_status_tests {
    use super::{RetrievalStatus, RetrievalStatusConversionError};

    #[test]
    fn converts_string_to_status() {
        let status = RetrievalStatus::try_from("retrieved".to_string()).unwrap();

        assert_eq!(status, RetrievalStatus::Retrieved);
    }

    #[test]
    fn rejects_unknown_status() {
        let error = RetrievalStatus::try_from("unknown").unwrap_err();

        assert_eq!(error, RetrievalStatusConversionError("unknown".to_string()));
    }

    #[test]
    fn converts_status_to_string_with_try_into() {
        let status: String = RetrievalStatus::Processed.try_into().unwrap();

        assert_eq!(status, "processed");
    }
}

//
//   GroupedSitemapContent
//

pub struct GroupedSitemapContent {
    pub id: Uuid,
    pub processed_sitemap_id: Uuid,
    pub retrieval_id: Uuid,
    pub retailer_code: RetailerCode,
    pub links: GroupedLinks,
}

impl GroupedSitemapContent {
    pub fn product_count(&self) -> usize {
        self.links.product_count()
    }
    pub fn catalog_count(&self) -> usize {
        self.links.catalog_count()
    }
    pub fn content_count(&self) -> usize {
        self.links.content_count()
    }
    pub fn not_interested_count(&self) -> usize {
        self.links.not_interested_count()
    }
    pub fn unknown_count(&self) -> usize {
        self.links.unknown_count()
    }
}

pub struct ProcessedSitemap {
    pub id: Uuid,
    pub retrieval_id: Uuid,
    pub retailer_code: RetailerCode,
    pub document: SitemapDocument,
    pub url_count: i32,
    pub processed_at: DateTime<Utc>,
}
