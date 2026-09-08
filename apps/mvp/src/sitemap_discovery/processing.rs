mod parse;
mod sitemap;

pub const PROCESS_SITEMAP_COMMAND: &str = "ProcessSitemap";
pub const SITEMAP_PROCESSED_EVENT: &str = "SitemapProcessed";

pub mod io {
    pub use super::eventing::SitemapProcessedSubscriber;
    pub use super::handler::ProcessSitemapHandler;
    pub use super::models::{ProcessSitemap, SitemapProcessed};
    pub use super::parse::SitemapParseError;
    pub use super::parse::{ChildRef, Parsed, parse};
    pub use super::repository::ProcessedSitemapRepository;
    pub use super::{PROCESS_SITEMAP_COMMAND, SITEMAP_PROCESSED_EVENT};
}

mod models {

    use crate::RepositoryError;
    use crate::sitemap_discovery::model::SitemapError;
    use bodul_shared::retailer::RetailerCode;
    use kernel::{ApplicationCommand, ApplicationEvent};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ProcessSitemap {
        pub retrieval_id: Uuid,
    }

    impl ApplicationCommand for ProcessSitemap {
        fn command_type(&self) -> &'static str {
            super::PROCESS_SITEMAP_COMMAND
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SitemapProcessed {
        pub retrieval_id: Uuid,
        pub processed_sitemap_id: Uuid,
        pub retailer_code: RetailerCode,
        pub url_count: usize,
    }

    impl ApplicationEvent for SitemapProcessed {
        fn event_type(&self) -> &'static str {
            super::SITEMAP_PROCESSED_EVENT
        }
    }

    #[derive(Debug, thiserror::Error)]
    pub enum ProcessSitemapError {
        #[error("sitemap: {0}")]
        Sitemap(#[from] SitemapError),

        #[error("processing: {0}")]
        RepositoryError(#[from] RepositoryError),
    }
}

mod handler {

    use super::super::io::RawSitemapDocumentRepository;
    use super::super::model::ProcessedSitemap;
    use super::super::retrieval::io::{SitemapRetrieval, SitemapRetrievalRepository};
    use super::model::NewProcessedSitemapRecord;
    use super::models::{ProcessSitemap, ProcessSitemapError, SitemapProcessed};
    use super::repository::ProcessedSitemapRepository;
    use crate::assembly::io::MvpEvent;
    use crate::lib_sitemap::io::RawSitemapDocument;
    use crate::sitemap_discovery::model::SitemapError;
    use crate::sitemap_discovery::processing::sitemap::document_from_raw;
    use crate::{IntoCommandError, RepositoryError};
    use ::retailer_sourcing::registry::sitemap_config;
    use bodul_shared::SitemapConfig;
    use bodul_shared::retailer::RetailerCode;
    use chrono::Utc;
    use kernel::io::{CommandError, CommandHandlerPort};
    use uuid::Uuid;

    type Error = ProcessSitemapError;
    pub struct ProcessSitemapHandler {
        retrieval_repo: SitemapRetrievalRepository,
        raw_sitemap_repo: RawSitemapDocumentRepository,
        processed_sitemap_repo: ProcessedSitemapRepository,
    }
    impl ProcessSitemapHandler {
        pub fn new(
            retrieval_repo: SitemapRetrievalRepository,
            raw_sitemap_repo: RawSitemapDocumentRepository,
            processed_sitemap_repo: ProcessedSitemapRepository,
        ) -> Self {
            Self {
                retrieval_repo,
                raw_sitemap_repo,
                processed_sitemap_repo,
            }
        }
    }

    impl CommandHandlerPort<ProcessSitemap, MvpEvent> for ProcessSitemapHandler {
        fn execute(&self, command: ProcessSitemap) -> Result<Vec<MvpEvent>, CommandError> {
            //TODO: make this error handling more nice :) (not all errors are storage related)
            self.handle(command).storage_err()
        }
    }

    impl ProcessSitemapHandler {
        fn load_retrieval(&self, retrieval_id: Uuid) -> Result<SitemapRetrieval, RepositoryError> {
            self.retrieval_repo.load(retrieval_id)
        }

        fn get_sitemap_config(retailer_code: &RetailerCode) -> Result<SitemapConfig, Error> {
            sitemap_config(retailer_code).ok_or_else(|| {
                let err = SitemapError::no_sitemap_config(*retailer_code);

                Error::Sitemap(err)
            })
        }

        fn load_raw_documents(&self, retrieval_id: Uuid) -> Result<Vec<RawSitemapDocument>, RepositoryError> {
            self.raw_sitemap_repo.load_by_retrieval(retrieval_id)
        }

        fn build_new_processed_sitemap(
            retrieval_id: Uuid,
            retailer_code: RetailerCode,
            config: SitemapConfig,
            raw_documents: &[RawSitemapDocument],
        ) -> Result<ProcessedSitemap, Error> {
            //TODO: figure out what to do with errors here.
            let (document, _errors) = document_from_raw(config, raw_documents)?;

            let url_count = document.all_urls(super::super::ROOT_SITEMAP_SOURCE).count();
            let url_count = i32::try_from(url_count).map_err(|_| SitemapError::TooManyUrls { count: url_count })?;

            let processed = ProcessedSitemap {
                id: Uuid::now_v7(),
                retrieval_id,
                retailer_code,
                document,
                url_count,
                processed_at: Utc::now(),
            };

            Ok(processed)
        }

        fn store_processed_sitemap(&self, record: NewProcessedSitemapRecord) -> Result<(), RepositoryError> {
            self.processed_sitemap_repo.store(record)?;

            Ok(())
        }

        fn mark_retrieval_processed(&self, retrieval_id: Uuid) -> Result<(), RepositoryError> {
            self.retrieval_repo.mark_processed(retrieval_id)?;

            Ok(())
        }

        fn handle(&self, command: ProcessSitemap) -> Result<Vec<MvpEvent>, Error> {
            let retrieval = self.load_retrieval(command.retrieval_id)?;
            let retailer_code = retrieval.retailer_code;

            let config = Self::get_sitemap_config(&retrieval.retailer_code)?;

            let raw_documents = self.load_raw_documents(command.retrieval_id)?;

            let processed =
                Self::build_new_processed_sitemap(command.retrieval_id, retailer_code, config, &raw_documents)?;

            let processed_sitemap_id = processed.id;
            let url_count = processed.url_count as usize;
            let record = Self::build_processed_sitemap_record(processed)?;

            self.store_processed_sitemap(record)?;
            self.mark_retrieval_processed(command.retrieval_id)?;

            let evt = SitemapProcessed {
                retrieval_id: command.retrieval_id,
                processed_sitemap_id,
                retailer_code,
                url_count,
            };

            Ok(vec![MvpEvent::SitemapProcessed(evt)])
        }

        fn build_processed_sitemap_record(
            processed: ProcessedSitemap,
        ) -> Result<NewProcessedSitemapRecord, RepositoryError> {
            let document = serde_json::to_value(&processed.document)?;
            let document_size = i32::try_from(document.to_string().len()).map_err(|_| {
                RepositoryError::Unexpected("processed sitemap document is too large for storage".to_string())
            })?;

            let record = NewProcessedSitemapRecord {
                id: processed.id,
                retrieval_id: processed.retrieval_id,
                retailer_code: processed.retailer_code.slug(),
                document,
                url_count: processed.url_count,
                document_size,
                processed_at: processed.processed_at,
            };

            Ok(record)
        }
    }
}

mod eventing {
    use crate::assembly::io::{AppCommand, MvpEvent};
    use crate::sitemap_discovery::io::GroupSitemapContent;
    use kernel::ApplicationCommand;
    use kernel::io::{CommandGateway, NewCommand, NewCommandEnvelope, NewCommandMetadata};
    use kernel::{EventError, EventSubscriberPort, NewEventEnvelope};
    use std::sync::Arc;
    use uuid::Uuid;

    pub struct SitemapProcessedSubscriber {
        command_gateway: Arc<CommandGateway>,
    }

    impl SitemapProcessedSubscriber {
        pub fn new(command_gateway: Arc<CommandGateway>) -> Self {
            Self { command_gateway }
        }
    }

    impl EventSubscriberPort for SitemapProcessedSubscriber {
        fn handle(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
            let event: MvpEvent = serde_json::from_str(&envelope.payload)
                .map_err(|error| EventError::SubscriberExecution(error.to_string()))?;
            let MvpEvent::SitemapProcessed(event) = event else {
                return Err(EventError::SubscriberExecution(
                    "unexpected event for sitemap grouping subscriber".to_string(),
                ));
            };

            let command = AppCommand::GroupSitemapContent(GroupSitemapContent {
                processed_sitemap_id: event.processed_sitemap_id,
            });
            let command_id = Uuid::now_v7();
            let gateway_envelope = NewCommandEnvelope {
                command: NewCommand {
                    command_type: command.command_type().to_string(),
                    payload: serde_json::to_string(&command)
                        .map_err(|error| EventError::SubscriberExecution(error.to_string()))?,
                },
                metadata: Some(NewCommandMetadata {
                    command_id,
                    correlation_id: envelope.metadata.as_ref().and_then(|metadata| metadata.correlation_id),
                    causation_id: envelope.metadata.as_ref().map(|metadata| metadata.event_id),
                    source: Some(format!("event:{}", super::SITEMAP_PROCESSED_EVENT)),
                }),
            };

            self.command_gateway
                .dispatch(gateway_envelope)
                .map_err(|error| EventError::SubscriberExecution(error.to_string()))
        }
    }
}

mod model {
    use super::super::model::ProcessedSitemap;
    use crate::RepositoryError;
    use crate::lib_sitemap::io::SitemapDocument;
    use crate::schema::processed_sitemaps;
    use bodul_shared::retailer::RetailerCode;
    use chrono::{DateTime, Utc};
    use diesel::{Insertable, Queryable, Selectable};
    use uuid::Uuid;

    #[derive(Debug, Clone, Queryable, Selectable)]
    #[diesel(table_name = processed_sitemaps)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct ProcessedSitemapRecord {
        pub id: Uuid,
        pub retrieval_id: Uuid,
        pub retailer_code: String,
        pub document: serde_json::Value,
        pub url_count: i32,
        pub processed_at: DateTime<Utc>,
    }

    impl TryFrom<ProcessedSitemapRecord> for ProcessedSitemap {
        type Error = RepositoryError;

        fn try_from(record: ProcessedSitemapRecord) -> Result<Self, Self::Error> {
            let retailer_code =
                RetailerCode::from_str(&record.retailer_code).map_err(RepositoryError::UnknownRetailerCode)?;
            let document: SitemapDocument = serde_json::from_value(record.document)?;

            let processed = ProcessedSitemap {
                id: record.id,
                retrieval_id: record.retrieval_id,
                retailer_code,
                document,
                url_count: record.url_count,
                processed_at: record.processed_at,
            };

            Ok(processed)
        }
    }

    #[derive(Insertable)]
    #[diesel(table_name = processed_sitemaps)]
    pub struct NewProcessedSitemapRecord {
        pub id: Uuid,
        pub retrieval_id: Uuid,
        pub retailer_code: String,
        pub document: serde_json::Value,
        pub url_count: i32,
        pub document_size: i32,
        pub processed_at: DateTime<Utc>,
    }
}

mod repository {
    use super::super::model::ProcessedSitemap;
    use super::model::{NewProcessedSitemapRecord, ProcessedSitemapRecord};
    use crate::RepositoryError;
    use crate::database::DbPool;
    use crate::schema::processed_sitemaps;
    use diesel::prelude::*;
    use uuid::Uuid;

    pub struct ProcessedSitemapRepository {
        pool: DbPool,
    }

    impl ProcessedSitemapRepository {
        pub fn new(pool: DbPool) -> Self {
            Self { pool }
        }

        pub fn store(&self, record: NewProcessedSitemapRecord) -> Result<(), RepositoryError> {
            let mut connection = self.pool.get()?;

            diesel::insert_into(processed_sitemaps::table)
                .values(record)
                .execute(&mut *connection)?;

            Ok(())
        }

        pub fn find_by_id(&self, processed_sitemap_id: Uuid) -> Result<ProcessedSitemap, RepositoryError> {
            let mut connection = self.pool.get()?;

            let record: ProcessedSitemapRecord = processed_sitemaps::table
                .find(processed_sitemap_id)
                .select(ProcessedSitemapRecord::as_select())
                .get_result(&mut *connection)?;

            record.try_into()
        }

        pub fn load_all(&self) -> Result<Vec<ProcessedSitemap>, RepositoryError> {
            let mut connection = self.pool.get()?;

            let records: Vec<ProcessedSitemapRecord> = processed_sitemaps::table
                .select(ProcessedSitemapRecord::as_select())
                .load(&mut *connection)?;

            records.into_iter().map(TryInto::try_into).collect()
        }
    }
}
