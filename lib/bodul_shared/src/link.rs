//! Link (page-URL) classification.
//!
//! Classifies the individual page links found inside a retailer's sitemaps.
//! Per-retailer rules live alongside each retailer's `sitemap_config` (see
//! [`crate::retailers`]); this module holds the shared [`LinkKind`] type and the
//! default Shopify rule reused by Shopify-based storefronts.

/// The type of a page link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinkKind {
    Product,
    Catalog,
    Content,
    NotInterested,
    Unknown,
}

impl LinkKind {
    /// Lowercase label.
    pub const fn as_str(self) -> &'static str {
        match self {
            LinkKind::Product => "product",
            LinkKind::Catalog => "catalog",
            LinkKind::Content => "content",
            LinkKind::NotInterested => "not_intersted",
            LinkKind::Unknown => "unknown",
        }
    }
}
