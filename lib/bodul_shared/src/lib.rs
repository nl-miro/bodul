pub mod link;
pub mod retailer;

#[derive(Debug, Clone)]
pub struct SitemapConfig {
    pub sitemap_url: Vec<String>,
}
