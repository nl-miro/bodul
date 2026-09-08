use bodul_shared::link::LinkKind;

/// Classifies Anker storefront URL paths. Most Anker stores use Shopify-style
/// plural paths; Korea also uses singular `/product/` paths.
pub fn anker_from_location(url: &str) -> LinkKind {
    let path = url.to_lowercase();
    if path.contains("/products/") || path.contains("/product/") {
        LinkKind::Product
    } else if path.contains("/collections/") || path.contains("/collection/") {
        LinkKind::Catalog
    } else if path.contains("/pages/") || path.contains("/blogs/") || path.contains("/blog/") {
        LinkKind::Content
    } else {
        LinkKind::Unknown
    }
}

pub mod au;
pub mod ca;
pub mod cn;
pub mod com;
pub mod de;
pub mod eu;
pub mod fr;
pub mod italycom;
pub mod japancom;
pub mod kr;
pub mod my;
pub mod nordicscom;
pub mod nz;
pub mod pl;
pub mod uk;
pub mod vn;
