use bodul_shared::link::LinkKind;

/// Classifies Shopify storefront URLs by their path conventions, ignoring case.
pub fn shopify_from_location(url: &str) -> LinkKind {
    let path = url.to_lowercase();
    if path.contains("/products/") {
        LinkKind::Product
    } else if path.contains("/collections/") {
        LinkKind::Catalog
    } else if path.contains("/pages/") || path.contains("/blogs/") {
        LinkKind::Content
    } else {
        LinkKind::Unknown
    }
}
