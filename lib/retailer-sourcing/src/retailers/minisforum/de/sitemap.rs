use bodul_shared::SitemapConfig;

/// Minisforum's European sitemap entry point used for German pages.
pub fn config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://minisforumpc.eu/sitemap.xml".to_string()],
    }
}
