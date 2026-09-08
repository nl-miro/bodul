use crate::retailers::anker::anker_from_location;
use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://www.anker.com.cn/sitemap.xml".to_string()],
    }
}

pub fn from_location(url: &str) -> LinkKind {
    anker_from_location(url)
}
