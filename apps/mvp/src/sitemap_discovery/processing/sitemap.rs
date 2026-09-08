use crate::lib_sitemap::io::{RawSitemapDocument, SitemapDocument};
use crate::sitemap_discovery::model::SitemapError;
use crate::sitemap_discovery::processing::sitemap::internal::SitemapDocumentFactory;
use bodul_shared::SitemapConfig;

pub fn document_from_raw(
    cfg: SitemapConfig,
    docs: &[RawSitemapDocument],
) -> Result<(SitemapDocument, Vec<SitemapError>), SitemapError> {
    // converting to vector
    let documents: Vec<RawSitemapDocument> = docs.to_vec();

    let mut factory = SitemapDocumentFactory::build(cfg, documents)?;

    factory.run()
}

mod internal {
    use super::super::super::MAX_SITEMAP_DEPTH;
    use super::super::parse::{ChildRef, Parsed, parse};
    use crate::lib_sitemap::io::{RawSitemapDocument, SitemapDocument};
    use crate::sitemap_discovery::model::SitemapError;
    use chrono::{DateTime, Utc};
    use bodul_shared::SitemapConfig;
    use std::collections::{HashMap, HashSet};

    /// A child sitemap referenced by an index.
    #[derive(Debug, Clone, PartialEq)]
    struct SitemapItem {
        url: String,
        last_modified: Option<DateTime<Utc>>,
        depth: usize,
    }
    impl SitemapItem {
        fn from_child(child: &ChildRef, depth: usize) -> Self {
            Self {
                url: child.location.to_string(),
                last_modified: child.last_modified,
                depth,
            }
        }

        fn initial(url: &str) -> Self {
            SitemapItem {
                url: url.to_string(),
                last_modified: None,
                depth: 0,
            }
        }
    }

    pub struct SitemapDocumentFactory {
        config: SitemapConfig,
        raw_documents_dict: HashMap<String, RawSitemapDocument>,
        active: HashSet<String>,
        errors: Vec<SitemapError>,
    }

    impl SitemapDocumentFactory {
        pub fn build(
            config: SitemapConfig,
            raw_documents: Vec<RawSitemapDocument>,
        ) -> Result<SitemapDocumentFactory, SitemapError> {
            ensure_roots_configured(&config)?;

            let raw_documents_dict = raw_documents
                .iter()
                .map(|document| (document.url.clone(), document.clone()))
                .collect::<HashMap<_, _>>();

            let active = HashSet::new();
            let errors = Vec::new();

            Ok(SitemapDocumentFactory {
                config,
                raw_documents_dict,
                active,
                errors,
            })
        }

        pub fn run(&mut self) -> Result<(SitemapDocument, Vec<SitemapError>), SitemapError> {
            let root_urls = self.config.sitemap_url.clone();

            let mut root_documents = Vec::new();
            for url in &root_urls {
                match self.build_document_node(SitemapItem::initial(url)) {
                    Ok(doc) => root_documents.push(doc),
                    Err(error) => {
                        self.errors.push(error.clone());
                    }
                }
            }

            let document = if root_documents.len() == 1 {
                root_documents.remove(0)
            } else {
                SitemapDocument {
                    children: root_documents,
                    ..SitemapDocument::default()
                }
            };

            Ok((document, self.errors.clone()))
        }

        fn build_document_node(&mut self, item: SitemapItem) -> Result<SitemapDocument, SitemapError> {
            ensure_depth(&item.url, item.depth)?;

            if !self.active.insert(item.url.to_string()) {
                let err = SitemapError::cyclic_reference(&item.url);
                return Err(err);
            }

            let raw = self
                .raw_documents_dict
                .get(&item.url)
                .ok_or_else(|| SitemapError::missing_raw_document(&item.url))?;

            let raw_last_modified = raw.last_modified;
            let parsed = parse(&raw.body, &item.url).map_err(|source| SitemapError::parse(&item.url, source))?;

            let mut urls = Vec::new();
            let mut children = Vec::new();

            match parsed {
                Parsed::UrlSet(url_set) => urls = url_set,
                Parsed::Index(child_refs) => {
                    for child in child_refs {
                        let sitemap_item = SitemapItem::from_child(&child, item.depth + 1);

                        let document = self.build_document_node(sitemap_item)?;

                        children.push(document);
                    }
                }
            }

            self.active.remove(&item.url);

            let location = Some(item.url.to_string());
            let last_modified = item.last_modified.or(raw_last_modified);

            let document = SitemapDocument::build(location, last_modified, urls, children);

            Ok(document)
        }
    }

    fn ensure_roots_configured(config: &SitemapConfig) -> Result<(), SitemapError> {
        if config.sitemap_url.is_empty() {
            return Err(SitemapError::NoRootSitemaps);
        }
        Ok(())
    }

    fn ensure_depth(url: &str, depth: usize) -> Result<(), SitemapError> {
        if depth > MAX_SITEMAP_DEPTH {
            return Err(SitemapError::MaximumDepthExceeded {
                url: url.to_string(),
                max_depth: MAX_SITEMAP_DEPTH,
            });
        }
        Ok(())
    }
}
