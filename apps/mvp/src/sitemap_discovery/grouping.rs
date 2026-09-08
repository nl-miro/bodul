pub const GROUP_SITEMAP_CONTENT_COMMAND: &str = "GroupSitemapContent";
pub const SITEMAP_CONTENT_GROUPED_EVENT: &str = "SitemapContentGrouped";

pub mod io {
    pub use super::handler::GroupSitemapContentHandler;
    pub use super::models::{GroupSitemapContent, SitemapContentGrouped};
    pub use super::repository::GroupedSitemapContentRepository;
    pub use super::{GROUP_SITEMAP_CONTENT_COMMAND, SITEMAP_CONTENT_GROUPED_EVENT};
}

mod models {
    use bodul_shared::retailer::RetailerCode;
    use kernel::{ApplicationCommand, ApplicationEvent};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GroupSitemapContent {
        pub processed_sitemap_id: Uuid,
    }

    impl ApplicationCommand for GroupSitemapContent {
        fn command_type(&self) -> &'static str {
            super::GROUP_SITEMAP_CONTENT_COMMAND
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SitemapContentGrouped {
        pub grouped_content_id: Uuid,
        pub processed_sitemap_id: Uuid,
        pub retrieval_id: Uuid,
        pub retailer_code: RetailerCode,
        pub product_count: usize,
        pub catalog_count: usize,
        pub content_count: usize,
        pub not_interested_count: usize,
        pub unknown_count: usize,
    }

    impl ApplicationEvent for SitemapContentGrouped {
        fn event_type(&self) -> &'static str {
            super::SITEMAP_CONTENT_GROUPED_EVENT
        }
    }
}

mod grouping {
    use bodul_shared::link::LinkKind;
    use bodul_shared::retailer::RetailerCode;
    use serde::{Deserialize, Serialize};

    use crate::lib_sitemap::io::SitemapDocument;
    use ::retailer_sourcing::registry::classify_link;

    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct GroupedLinks {
        pub product: Vec<String>,
        pub catalog: Vec<String>,
        pub content: Vec<String>,
        #[serde(rename = "not_interested")]
        pub not_interested: Vec<String>,
        pub unknown: Vec<String>,
    }

    impl GroupedLinks {
        pub fn from_document(retailer_code: RetailerCode, document: &SitemapDocument) -> Self {
            let mut links = Self::default();
            for url in document.all_urls(super::super::ROOT_SITEMAP_SOURCE) {
                match classify_link(retailer_code, &url.location, &url.source, url.images.len()) {
                    LinkKind::Product => links.product.push(url.location),
                    LinkKind::Catalog => links.catalog.push(url.location),
                    LinkKind::Content => links.content.push(url.location),
                    LinkKind::NotInterested => links.not_interested.push(url.location),
                    LinkKind::Unknown => links.unknown.push(url.location),
                }
            }
            dedup(&mut links.product);
            dedup(&mut links.catalog);
            dedup(&mut links.content);
            dedup(&mut links.not_interested);
            dedup(&mut links.unknown);
            links
        }

        pub fn product_count(&self) -> usize {
            self.product.len()
        }

        pub fn catalog_count(&self) -> usize {
            self.catalog.len()
        }

        pub fn content_count(&self) -> usize {
            self.content.len()
        }
        pub fn not_interested_count(&self) -> usize {
            self.not_interested.len()
        }
        pub fn unknown_count(&self) -> usize {
            self.unknown.len()
        }
    }

    fn dedup(list: &mut Vec<String>) {
        list.sort_unstable();
        list.dedup();
    }

    #[cfg(test)]
    mod tests {
        use super::GroupedLinks;

        #[test]
        fn round_trips_product_urls_through_json() {
            let mut links = GroupedLinks::default();
            links.product = vec![
                "https://example.com/products/a".to_string(),
                "https://example.com/products/b".to_string(),
            ];
            links.catalog = vec!["https://example.com/collections/x".to_string()];

            let value = serde_json::to_value(&links).unwrap();
            let restored: GroupedLinks = serde_json::from_value(value).unwrap();

            assert_eq!(restored.product, links.product);
            assert_eq!(restored.catalog, links.catalog);
        }
    }
}

pub use grouping::GroupedLinks;

mod handler {
    use super::super::model::{GroupedSitemapContent, ProcessedSitemap};
    use super::super::processing::io::ProcessedSitemapRepository;
    use super::super::retrieval::io::SitemapRetrievalRepository;
    use super::grouping::GroupedLinks;
    use super::model::NewGroupedSitemapContentRecord;
    use super::models::{GroupSitemapContent, SitemapContentGrouped};
    use super::repository::GroupedSitemapContentRepository;
    use crate::IntoCommandError;
    use crate::RepositoryError;
    use crate::assembly::io::MvpEvent;
    use kernel::io::{CommandError, CommandHandlerPort};
    use uuid::Uuid;

    pub struct GroupSitemapContentHandler {
        retrieval_repo: SitemapRetrievalRepository,
        processed_sitemap_repo: ProcessedSitemapRepository,
        grouped_content_repo: GroupedSitemapContentRepository,
    }

    impl GroupSitemapContentHandler {
        pub fn new(
            retrieval_repo: SitemapRetrievalRepository,
            processed_sitemap_repo: ProcessedSitemapRepository,
            grouped_content_repo: GroupedSitemapContentRepository,
        ) -> Self {
            Self {
                retrieval_repo,
                processed_sitemap_repo,
                grouped_content_repo,
            }
        }

        fn load_processed_sitemap(&self, id: Uuid) -> Result<ProcessedSitemap, CommandError> {
            self.processed_sitemap_repo.find_by_id(id).storage_err()
        }

        fn build_grouped_content(processed: &ProcessedSitemap) -> GroupedSitemapContent {
            let document = &processed.document;
            GroupedSitemapContent {
                id: Uuid::now_v7(),
                processed_sitemap_id: processed.id,
                retrieval_id: processed.retrieval_id,
                retailer_code: processed.retailer_code,
                links: GroupedLinks::from_document(processed.retailer_code, document),
            }
        }

        fn store_grouped_content(&self, record: NewGroupedSitemapContentRecord) -> Result<(), CommandError> {
            self.grouped_content_repo.store(record).storage_err()
        }

        fn mark_retrieval_grouped(&self, retrieval_id: Uuid) -> Result<(), CommandError> {
            self.retrieval_repo.mark_grouped(retrieval_id).storage_err()
        }

        fn build_event(grouped_content: &GroupedSitemapContent) -> SitemapContentGrouped {
            SitemapContentGrouped {
                grouped_content_id: grouped_content.id,
                processed_sitemap_id: grouped_content.processed_sitemap_id,
                retrieval_id: grouped_content.retrieval_id,
                retailer_code: grouped_content.retailer_code,
                product_count: grouped_content.links.product.len(),
                catalog_count: grouped_content.links.catalog.len(),
                content_count: grouped_content.links.content.len(),
                not_interested_count: grouped_content.links.not_interested.len(),
                unknown_count: grouped_content.links.unknown.len(),
            }
        }
    }

    impl CommandHandlerPort<GroupSitemapContent, MvpEvent> for GroupSitemapContentHandler {
        fn execute(&self, command: GroupSitemapContent) -> Result<Vec<MvpEvent>, CommandError> {
            let processed = self.load_processed_sitemap(command.processed_sitemap_id)?;
            let grouped_content = Self::build_grouped_content(&processed);
            let record = build_grouped_content_record(&grouped_content)?;
            self.store_grouped_content(record)?;
            self.mark_retrieval_grouped(grouped_content.retrieval_id)?;
            let event = Self::build_event(&grouped_content);
            Ok(vec![MvpEvent::SitemapContentGrouped(event)])
        }
    }

    fn build_grouped_content_record(
        grouped_content: &GroupedSitemapContent,
    ) -> Result<NewGroupedSitemapContentRecord, CommandError> {
        let content = serde_json::to_value(&grouped_content.links)
            .map_err(RepositoryError::from)
            .storage_err()?;
        let content_size = i32::try_from(content.to_string().len())
            .map_err(|_| RepositoryError::Unexpected("grouped sitemap content is too large for storage".to_string()))
            .storage_err()?;

        let record = NewGroupedSitemapContentRecord {
            id: grouped_content.id,
            processed_sitemap_id: grouped_content.processed_sitemap_id,
            retrieval_id: grouped_content.retrieval_id,
            retailer_code: grouped_content.retailer_code.slug().to_string(),
            content,
            product_count: stored_count(grouped_content.product_count())?,
            catalog_count: stored_count(grouped_content.catalog_count())?,
            content_count: stored_count(grouped_content.content_count())?,
            not_interested_count: stored_count(grouped_content.not_interested_count())?,
            unknown_count: stored_count(grouped_content.unknown_count())?,
            content_size,
            grouped_at: chrono::Utc::now(),
        };

        Ok(record)
    }

    fn stored_count(count: usize) -> Result<i32, CommandError> {
        i32::try_from(count)
            .map_err(|_| {
                RepositoryError::Unexpected(format!("grouped sitemap count is too large for storage: {count}"))
            })
            .storage_err()
    }
}

pub mod model {
    use super::super::model::GroupedSitemapContent;
    use crate::RepositoryError;
    use crate::schema::grouped_sitemap_contents;
    use bodul_shared::retailer::RetailerCode;
    use chrono::{DateTime, Utc};
    use diesel::{Insertable, Queryable, Selectable};
    use uuid::Uuid;

    #[derive(Insertable)]
    #[diesel(table_name = grouped_sitemap_contents)]
    pub struct NewGroupedSitemapContentRecord {
        pub id: Uuid,
        pub processed_sitemap_id: Uuid,
        pub retrieval_id: Uuid,
        pub retailer_code: String,
        pub content: serde_json::Value,
        pub product_count: i32,
        pub catalog_count: i32,
        pub content_count: i32,
        pub not_interested_count: i32,
        pub unknown_count: i32,
        pub content_size: i32,
        pub grouped_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, Queryable, Selectable)]
    #[diesel(table_name = grouped_sitemap_contents)]
    pub struct GroupedSitemapContentRecord {
        pub id: Uuid,
        pub processed_sitemap_id: Uuid,
        pub retrieval_id: Uuid,
        pub retailer_code: String,
        pub content: serde_json::Value,
    }

    impl TryFrom<GroupedSitemapContentRecord> for GroupedSitemapContent {
        type Error = RepositoryError;

        fn try_from(record: GroupedSitemapContentRecord) -> Result<Self, Self::Error> {
            let retailer_code =
                RetailerCode::from_str(&record.retailer_code).map_err(RepositoryError::UnknownRetailerCode)?;
            let links = serde_json::from_value(record.content)?;

            Ok(GroupedSitemapContent {
                id: record.id,
                processed_sitemap_id: record.processed_sitemap_id,
                retrieval_id: record.retrieval_id,
                retailer_code,
                links,
            })
        }
    }
}

pub mod repository {
    use super::super::model::GroupedSitemapContent;
    use super::model::{GroupedSitemapContentRecord, NewGroupedSitemapContentRecord};
    use crate::RepositoryError;
    use crate::database::DbPool;
    use crate::schema::grouped_sitemap_contents;
    use diesel::prelude::*;
    use uuid::Uuid;

    pub struct GroupedSitemapContentRepository {
        pool: DbPool,
    }

    impl GroupedSitemapContentRepository {
        pub fn new(pool: DbPool) -> Self {
            Self { pool }
        }

        pub fn store(&self, record: NewGroupedSitemapContentRecord) -> Result<(), RepositoryError> {
            let mut connection = self.pool.get()?;

            diesel::insert_into(grouped_sitemap_contents::table)
                .values(record)
                .execute(&mut *connection)?;

            Ok(())
        }

        /// Load a grouped sitemap content record as a domain entity, including its classified links.
        pub fn load(&self, id: Uuid) -> Result<GroupedSitemapContent, RepositoryError> {
            let mut connection = self.pool.get()?;

            let record: GroupedSitemapContentRecord = grouped_sitemap_contents::table
                .find(id)
                .select(GroupedSitemapContentRecord::as_select())
                .get_result(&mut *connection)?;

            record.try_into()
        }
    }
}
