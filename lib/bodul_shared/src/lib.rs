pub mod link;
pub mod retailer;

#[derive(Debug, Clone)]
pub struct SitemapConfig {
    pub sitemap_url: Vec<String>,
}

pub mod prelude {
    pub use super::NormalizedData;
    pub use super::retailer::{RetailerCode, RetailerCodeConversionError};
}

pub struct NormalizedData {
    pub document_json: Option<String>,
    pub recommendation_offers: Vec<String>,
    pub upsell_offers: Vec<String>,
}

impl NormalizedData {
    pub fn just_document_json(document_json: impl Into<Option<String>>) -> Self {
        NormalizedData {
            document_json: document_json.into(),
            recommendation_offers: vec![],
            upsell_offers: vec![],
        }
    }
}
