use crate::RecordMappingError;
use crate::sitemap_discovery::model::RetrievalStatus;
use bodul_shared::retailer::RetailerCode;

pub const REQUEST_SITEMAP_RETRIEVAL_COMMAND: &str = "RequestSitemapRetrieval";
pub const SITEMAP_RETRIEVED_EVENT: &str = "SitemapRetrieved";

pub mod io {
    // command, event and error types
    pub use super::models::{RequestSitemapRetrieval, RequestSitemapRetrievalError, SitemapRetrieved};
    pub use super::{REQUEST_SITEMAP_RETRIEVAL_COMMAND, SITEMAP_RETRIEVED_EVENT};

    // command handler
    pub use super::handler::RequestSitemapRetrievalHandler;

    // event subscriber
    pub use super::eventing::SitemapRetrievedSubscriber;

    pub use super::entity::SitemapRetrieval;

    pub use super::infra_sitemap_retrieval_repository::SitemapRetrievalRepository;

    pub use super::infra_sitemap_document_repository::RawSitemapDocumentRepository;
}

mod models {
    use derive_new::new;

    use crate::RepositoryError;
    use crate::sitemap_discovery::model::SitemapError;
    use kernel::{ApplicationCommand, ApplicationEvent};
    use serde::{Deserialize, Serialize};
    use bodul_shared::retailer::RetailerCode;
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize, new)]
    pub struct RequestSitemapRetrieval {
        pub retrieval_id: Uuid,
        pub retailer_code: RetailerCode,
    }

    impl ApplicationCommand for RequestSitemapRetrieval {
        fn command_type(&self) -> &'static str {
            super::REQUEST_SITEMAP_RETRIEVAL_COMMAND
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, new)]
    pub struct SitemapRetrieved {
        pub retrieval_id: Uuid,
        pub retailer_code: RetailerCode,
        pub document_count: usize,
    }

    impl ApplicationEvent for SitemapRetrieved {
        fn event_type(&self) -> &'static str {
            super::SITEMAP_RETRIEVED_EVENT
        }
    }

    #[derive(Debug, thiserror::Error)]
    pub enum RequestSitemapRetrievalError {
        #[error("sitemap: {0}")]
        Sitemap(#[from] SitemapError),

        #[error("retrieval: {0}")]
        RepositoryError(#[from] RepositoryError),

        #[error("sitemap retrieval failed ({sitemap}) and recording the failure also failed ({repository})")]
        FailedAndCouldNotRecord {
            sitemap: SitemapError,
            repository: RepositoryError,
        },
    }
}

mod handler {
    // TODO: should this be part of assembly?
    use crate::assembly::io::MvpEvent;
    use derive_new::new;

    use super::fetching::fetch;
    use super::infra_sitemap_document_model::NewRawSitemapDocument;
    use super::infra_sitemap_document_repository::RawSitemapDocumentRepository;
    use super::infra_sitemap_retrieval_repository::SitemapRetrievalRepository;
    use super::models::{RequestSitemapRetrieval, RequestSitemapRetrievalError, SitemapRetrieved};
    use crate::lib_sitemap::io::RawSitemapDocument;
    use crate::sitemap_discovery::model::SitemapError;
    use crate::{IntoCommandError, RepositoryError};
    use kernel::io::{CommandError, CommandHandlerPort};

    type Error = RequestSitemapRetrievalError;
    type Command = RequestSitemapRetrieval;

    #[derive(new)]
    pub struct RequestSitemapRetrievalHandler {
        retrieval_repo: SitemapRetrievalRepository,
        raw_sitemap_repo: RawSitemapDocumentRepository,
    }

    impl CommandHandlerPort<RequestSitemapRetrieval, MvpEvent> for RequestSitemapRetrievalHandler {
        fn execute(&self, cmd: RequestSitemapRetrieval) -> Result<Vec<MvpEvent>, CommandError> {
            //TODO: make this error handling more nice :) (not all errors are storage related)
            self.handle(cmd).storage_err()
        }
    }

    impl RequestSitemapRetrievalHandler {
        fn handle(&self, cmd: Command) -> Result<Vec<MvpEvent>, Error> {
            self.record_request(&cmd)?;

            let documents = match fetch(&cmd.retailer_code) {
                Ok(documents) => documents,
                Err(sitemap) => {
                    if let Err(repository) = self.mark_as_failed(&cmd, &sitemap) {
                        return Err(Error::FailedAndCouldNotRecord { sitemap, repository });
                    }
                    return Err(sitemap.into());
                }
            };

            let records = Self::build_document_records(&cmd, &documents)?;
            self.store_documents(&records)?;
            self.confirm_request_retrieved(&cmd)?;

            let evt = SitemapRetrieved::new(cmd.retrieval_id, cmd.retailer_code, documents.len());

            Ok(vec![MvpEvent::SitemapRetrieved(evt)])
        }

        fn record_request(&self, cmd: &Command) -> Result<(), RepositoryError> {
            self.retrieval_repo
                .insert(cmd.retrieval_id, cmd.retailer_code)
                .map_err(|e| {
                    RepositoryError::Unexpected(format!(
                        "[{:?}] failed to insert retrieval '{}': {}",
                        cmd.retailer_code, cmd.retrieval_id, e
                    ))
                })?;

            Ok(())
        }

        fn mark_as_failed(&self, cmd: &Command, error: &SitemapError) -> Result<(), RepositoryError> {
            self.retrieval_repo.mark_failed(cmd.retrieval_id, &error.to_string())?;

            Ok(())
        }

        fn build_document_records<'a>(
            cmd: &Command,
            documents: &'a [RawSitemapDocument],
        ) -> Result<Vec<NewRawSitemapDocument<'a>>, RepositoryError> {
            let fetched_at = chrono::Utc::now();
            documents
                .iter()
                .map(|document| {
                    let body_size = i32::try_from(document.body_size).map_err(|_| {
                        RepositoryError::Unexpected(format!(
                            "raw sitemap body is too large for storage: {} bytes",
                            document.body_size
                        ))
                    })?;

                    let record = NewRawSitemapDocument {
                        id: uuid::Uuid::now_v7(),
                        retrieval_id: cmd.retrieval_id,
                        url: &document.url,
                        last_modified: document.last_modified,
                        body: &document.body,
                        body_size,
                        fetched_at,
                    };

                    Ok(record)
                })
                .collect()
        }

        fn store_documents(&self, records: &[NewRawSitemapDocument<'_>]) -> Result<(), RepositoryError> {
            self.raw_sitemap_repo.store(records)?;

            Ok(())
        }

        fn confirm_request_retrieved(&self, cmd: &Command) -> Result<(), RepositoryError> {
            self.retrieval_repo.mark_retrieved(cmd.retrieval_id)?;

            Ok(())
        }
    }
}

mod eventing {
    use super::super::processing::io::ProcessSitemap;
    use crate::assembly::io::{AppCommand, MvpEvent};
    use kernel::ApplicationCommand;
    use kernel::io::{CommandGateway, NewCommand, NewCommandEnvelope, NewCommandMetadata};
    use kernel::{EventError, EventSubscriberPort, NewEventEnvelope};
    use std::sync::Arc;
    use uuid::Uuid;

    pub struct SitemapRetrievedSubscriber {
        command_gateway: Arc<CommandGateway>,
    }

    impl SitemapRetrievedSubscriber {
        pub fn new(command_gateway: Arc<CommandGateway>) -> Self {
            Self { command_gateway }
        }
    }

    impl EventSubscriberPort for SitemapRetrievedSubscriber {
        fn handle(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
            let event: MvpEvent = serde_json::from_str(&envelope.payload)
                .map_err(|error| EventError::SubscriberExecution(error.to_string()))?;
            let MvpEvent::SitemapRetrieved(event) = event else {
                return Err(EventError::SubscriberExecution(
                    "unexpected event for sitemap processing subscriber".to_string(),
                ));
            };

            let command = AppCommand::ProcessSitemap(ProcessSitemap {
                retrieval_id: event.retrieval_id,
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
                    source: Some(format!("event:{}", super::SITEMAP_RETRIEVED_EVENT)),
                }),
            };

            self.command_gateway
                .dispatch(gateway_envelope)
                .map_err(|error| EventError::SubscriberExecution(error.to_string()))
        }
    }
}

mod entity {
    use super::super::model::RetrievalStatus;
    use bodul_shared::retailer::RetailerCode;
    use uuid::Uuid;

    pub struct SitemapRetrieval {
        pub id: Uuid,
        pub retailer_code: RetailerCode,
        pub status: RetrievalStatus,
    }
}

mod infra_sitemap_retrieval_model {
    use super::super::model::RetrievalStatus;
    use super::entity::SitemapRetrieval;
    use super::{map_to_retrieval_code, map_to_retrieval_status};
    use crate::RecordMappingError;
    use crate::schema::sitemap_retrievals;
    use chrono::{DateTime, Utc};
    use diesel::{Insertable, Queryable, Selectable};
    use bodul_shared::retailer::RetailerCode;
    use uuid::Uuid;

    #[derive(Insertable)]
    #[diesel(table_name = sitemap_retrievals)]
    pub struct NewSitemapRetrievalRecord {
        pub id: Uuid,
        pub retailer_code: String,
        pub status: &'static str,
        pub requested_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, Queryable, Selectable)]
    #[diesel(table_name = sitemap_retrievals)]
    pub struct SitemapRetrievalRecord {
        pub id: Uuid,
        pub retailer_code: String,
        pub status: String,
        pub requested_at: DateTime<Utc>,
        pub retrieved_at: Option<DateTime<Utc>>,
        pub processed_at: Option<DateTime<Utc>>,
        pub grouped_at: Option<DateTime<Utc>>,
        pub error: Option<String>,
    }

    impl From<(Uuid, RetailerCode)> for NewSitemapRetrievalRecord {
        fn from((id, retailer_code): (Uuid, RetailerCode)) -> Self {
            Self {
                id,
                retailer_code: retailer_code.slug().to_string(),
                status: RetrievalStatus::Requested.as_str(),
                requested_at: Utc::now(),
            }
        }
    }

    impl TryFrom<SitemapRetrievalRecord> for SitemapRetrieval {
        type Error = RecordMappingError;

        fn try_from(record: SitemapRetrievalRecord) -> Result<Self, Self::Error> {
            let retailer_code = map_to_retrieval_code(record.retailer_code.as_str())?;
            let status = map_to_retrieval_status(record.status.as_str())?;

            let retrieval = SitemapRetrieval {
                id: record.id,
                retailer_code,
                status,
            };

            Ok(retrieval)
        }
    }
}

/// Maps a stored retailer-code string onto its typed code, preserving the conversion
/// error so callers can match on it.
fn map_to_retrieval_code(retailer_code: &str) -> Result<RetailerCode, RecordMappingError> {
    Ok(RetailerCode::try_from(retailer_code)?)
}

/// Maps a stored status string onto [`RetrievalStatus`], preserving the conversion error.
fn map_to_retrieval_status(status: &str) -> Result<RetrievalStatus, RecordMappingError> {
    Ok(RetrievalStatus::try_from(status)?)
}

mod infra_sitemap_retrieval_repository {
    use crate::database::DbPool;
    use crate::schema::sitemap_retrievals;
    use crate::schema::sitemap_retrievals::{grouped_at, processed_at, retrieved_at, status};

    use super::super::model::RetrievalStatus;
    use super::entity::SitemapRetrieval;
    use super::infra_sitemap_retrieval_model::{NewSitemapRetrievalRecord, SitemapRetrievalRecord};
    use crate::RepositoryError;
    use diesel::RunQueryDsl;
    use diesel::prelude::*;
    use bodul_shared::retailer::RetailerCode;
    use uuid::Uuid;

    pub struct SitemapRetrievalRepository {
        pool: DbPool,
    }

    impl SitemapRetrievalRepository {
        pub fn new(pool: DbPool) -> Self {
            Self { pool }
        }

        pub fn load(&self, retrieval_id: Uuid) -> Result<SitemapRetrieval, RepositoryError> {
            let mut connection = self.pool.get()?;
            let record: SitemapRetrievalRecord = sitemap_retrievals::table
                .find(retrieval_id)
                .get_result(&mut *connection)?;

            let result = record.try_into()?;

            Ok(result)
        }

        pub fn insert(
            &self,
            retrieval_id: Uuid,
            retailer_code: RetailerCode,
        ) -> Result<SitemapRetrieval, RepositoryError> {
            let mut connection = self.pool.get()?;

            let new_record: NewSitemapRetrievalRecord = (retrieval_id, retailer_code).into();

            let record: SitemapRetrievalRecord = diesel::insert_into(sitemap_retrievals::table)
                .values(new_record)
                .returning(SitemapRetrievalRecord::as_returning())
                .get_result(&mut *connection)?;

            let res: SitemapRetrieval = record.try_into()?;

            Ok(res)
        }

        pub fn mark_retrieved(&self, retrieval_id: Uuid) -> Result<(), RepositoryError> {
            let mut connection = self.pool.get()?;

            let updated = diesel::update(
                sitemap_retrievals::table
                    .find(retrieval_id)
                    .filter(status.eq_any([RetrievalStatus::Requested.as_str(), RetrievalStatus::Failed.as_str()])),
            )
            .set((
                status.eq(RetrievalStatus::Retrieved.as_str()),
                retrieved_at.eq(diesel::dsl::now),
                sitemap_retrievals::columns::error.eq(None::<String>),
            ))
            .execute(&mut *connection)?;

            Self::ensure_transitioned(retrieval_id, RetrievalStatus::Requested, updated)
        }

        pub fn mark_failed(&self, retrieval_id: Uuid, error: &str) -> Result<(), RepositoryError> {
            let mut connection = self.pool.get()?;

            // A retrieval can fail at any stage, so this is not filtered on status.
            let updated = diesel::update(sitemap_retrievals::table.find(retrieval_id))
                .set((
                    status.eq(RetrievalStatus::Failed.as_str()),
                    sitemap_retrievals::columns::error.eq(error),
                ))
                .execute(&mut *connection)?;

            Self::ensure_exists(retrieval_id, updated)
        }

        pub fn mark_processed(&self, retrieval_id: Uuid) -> Result<(), RepositoryError> {
            let mut connection = self.pool.get()?;
            let updated = diesel::update(
                sitemap_retrievals::table
                    .find(retrieval_id)
                    .filter(status.eq_any([RetrievalStatus::Retrieved.as_str(), RetrievalStatus::Failed.as_str()])),
            )
            .set((
                status.eq(RetrievalStatus::Processed.as_str()),
                processed_at.eq(diesel::dsl::now),
                sitemap_retrievals::columns::error.eq(None::<String>),
            ))
            .execute(&mut *connection)?;

            Self::ensure_transitioned(retrieval_id, RetrievalStatus::Retrieved, updated)
        }

        pub fn mark_grouped(&self, retrieval_id: Uuid) -> Result<(), RepositoryError> {
            let mut connection = self.pool.get()?;
            let updated = diesel::update(
                sitemap_retrievals::table
                    .find(retrieval_id)
                    .filter(status.eq_any([RetrievalStatus::Processed.as_str(), RetrievalStatus::Failed.as_str()])),
            )
            .set((
                status.eq(RetrievalStatus::Grouped.as_str()),
                grouped_at.eq(diesel::dsl::now),
                sitemap_retrievals::columns::error.eq(None::<String>),
            ))
            .execute(&mut *connection)?;

            Self::ensure_transitioned(retrieval_id, RetrievalStatus::Processed, updated)
        }

        fn ensure_exists(retrieval_id: Uuid, updated: usize) -> Result<(), RepositoryError> {
            if updated == 1 {
                return Ok(());
            }

            Err(RepositoryError::Unexpected(format!(
                "retrieval {retrieval_id} not found"
            )))
        }

        fn ensure_transitioned(
            retrieval_id: Uuid,
            expected_status: RetrievalStatus,
            updated: usize,
        ) -> Result<(), RepositoryError> {
            if updated == 1 {
                return Ok(());
            }

            Err(RepositoryError::Unexpected(format!(
                "retrieval {retrieval_id} was not in expected '{}' state",
                expected_status.as_str()
            )))
        }
    }
}
mod infra_sitemap_document_model {
    use crate::schema::raw_sitemap_documents;
    use chrono::{DateTime, Utc};
    use diesel::{Insertable, Queryable, Selectable};
    use uuid::Uuid;

    #[derive(Insertable)]
    #[diesel(table_name = raw_sitemap_documents)]
    pub struct NewRawSitemapDocument<'a> {
        pub id: Uuid,
        pub retrieval_id: Uuid,
        pub url: &'a str,
        pub last_modified: Option<DateTime<Utc>>,
        pub body: &'a str,
        pub body_size: i32,
        pub fetched_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, Queryable, Selectable)]
    #[diesel(table_name = raw_sitemap_documents)]
    pub struct RawSitemapDocumentRecord {
        pub url: String,
        pub last_modified: Option<DateTime<Utc>>,
        pub body: String,
        pub body_size: i32,
    }
}

mod infra_sitemap_document_repository {
    use super::infra_sitemap_document_model::{NewRawSitemapDocument, RawSitemapDocumentRecord};
    use crate::RepositoryError;
    use crate::database::DbPool;
    use crate::lib_sitemap::io::RawSitemapDocument;
    use crate::schema::raw_sitemap_documents;
    use diesel::RunQueryDsl;
    use diesel::prelude::*;
    use uuid::Uuid;

    pub struct RawSitemapDocumentRepository {
        pool: DbPool,
    }

    impl RawSitemapDocumentRepository {
        pub fn new(pool: DbPool) -> Self {
            Self { pool }
        }

        pub fn store(&self, records: &[NewRawSitemapDocument<'_>]) -> Result<(), RepositoryError> {
            let mut connection = self.pool.get()?;

            diesel::insert_into(raw_sitemap_documents::table)
                .values(records)
                .execute(&mut *connection)?;
            Ok(())
        }

        pub fn load_by_retrieval(&self, retrieval_id: Uuid) -> Result<Vec<RawSitemapDocument>, RepositoryError> {
            let mut connection = self.pool.get()?;

            let records = raw_sitemap_documents::table
                .filter(raw_sitemap_documents::retrieval_id.eq(retrieval_id))
                .order(raw_sitemap_documents::fetched_at.asc())
                .select(RawSitemapDocumentRecord::as_select())
                .load(&mut *connection)?;

            Ok(records
                .into_iter()
                .map(|r| RawSitemapDocument {
                    url: r.url,
                    last_modified: r.last_modified,
                    body: r.body,
                    body_size: r.body_size as usize,
                })
                .collect())
        }
    }
}

mod fetching {
    use super::super::MAX_SITEMAP_DEPTH;
    use crate::lib_sitemap::io::RawSitemapDocument;
    use crate::retailer_data_ingestion::{Client, FetchError};
    use crate::sitemap_discovery::io::{ChildRef, Parsed, parse};
    use crate::sitemap_discovery::model::SitemapError;
    use ::retailer_sourcing::registry::sitemap_config;
    use chrono::{DateTime, Utc};
    use bodul_shared::SitemapConfig;
    use bodul_shared::retailer::RetailerCode;
    use std::collections::HashSet;

    pub fn fetch(retailer: &RetailerCode) -> Result<Vec<RawSitemapDocument>, SitemapError> {
        let config = get_config(retailer)?;

        let mut factory = SitemapFetchResultFactory {
            getter: Client::get,
            already_fetched: HashSet::new(),
            active: HashSet::new(),
            errors: vec![],
        };

        let result = factory.fetch_and_build(config)?;

        Ok(result.documents)
    }

    pub struct SitemapFetchResultFactory {
        getter: fn(url: &str) -> Result<String, FetchError>,
        already_fetched: HashSet<String>,
        active: HashSet<String>,
        errors: Vec<SitemapError>,
    }

    pub struct SitemapFetchResult {
        documents: Vec<RawSitemapDocument>,
        errors: Vec<SitemapError>,
    }

    /// A child sitemap referenced by an index.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SitemapItem {
        pub location: String,
        pub last_modified: Option<DateTime<Utc>>,
        pub depth: usize,
    }

    impl SitemapItem {
        fn from_child(child: &ChildRef, depth: usize) -> Self {
            Self {
                location: child.location.to_string(),
                last_modified: child.last_modified,
                depth,
            }
        }

        fn initial(url: &str) -> Self {
            SitemapItem {
                location: url.to_string(),
                last_modified: None,
                depth: 0,
            }
        }
    }

    impl SitemapFetchResultFactory {
        pub fn fetch_and_build(&mut self, sitemap_cfg: SitemapConfig) -> Result<SitemapFetchResult, SitemapError> {
            use std::collections::VecDeque;

            let mut documents = vec![];

            let mut queue: VecDeque<SitemapItem> = sitemap_cfg
                .sitemap_url
                .iter()
                .map(|url| SitemapItem::initial(url))
                .collect();

            while let Some(i) = queue.pop_front() {
                if let Some(document) = self.fetch_sitemap(&i)? {
                    documents.push(document.clone());

                    let parsed = match parse_it(&i.location, &document.body) {
                        Ok(parsed) => parsed,
                        Err(error) => {
                            self.add_error(error);
                            continue;
                        }
                    };

                    if let Parsed::Index(children) = parsed {
                        for child in children {
                            let sitemap_item = SitemapItem::from_child(&child, i.depth + 1);

                            queue.push_back(sitemap_item);
                        }
                    }
                }
            }

            Ok(SitemapFetchResult {
                documents,
                errors: self.errors.clone(),
            })
        }

        fn fetch_sitemap(&mut self, i: &SitemapItem) -> Result<Option<RawSitemapDocument>, SitemapError> {
            if i.depth > MAX_SITEMAP_DEPTH {
                let err = SitemapError::maximum_depth_exceeded(i.location.to_string(), MAX_SITEMAP_DEPTH);
                self.add_error(err);
                return Ok(None);
            }

            if self.already_fetched(i.location.as_str()) {
                let err = SitemapError::AlreadyFetched {
                    url: i.location.to_string(),
                };
                self.add_error(err);
                return Ok(None);
            }

            if !self.mark_as_activated(&i.location) {
                let err = SitemapError::cyclic_reference(&i.location);
                self.add_error(err);

                return Ok(None);
            }

            let document = match self.fetch_file(i) {
                Ok(document) => document,
                Err(error) => {
                    self.add_error(error);
                    return Ok(None);
                }
            };

            self.mark_as_processed(i.location.as_str());

            Ok(Some(document))
        }
        fn fetch_file(&self, i: &SitemapItem) -> Result<RawSitemapDocument, SitemapError> {
            let body = (self.getter)(&i.location).map_err(|source| SitemapError::Fetch {
                url: i.location.to_string(),
                source,
            })?;

            let document = RawSitemapDocument {
                url: i.location.to_string(),
                last_modified: i.last_modified,
                body_size: body.len(),
                body,
            };

            Ok(document)
        }

        fn already_fetched(&mut self, url: &str) -> bool {
            self.already_fetched.contains(url)
        }
        fn mark_as_activated(&mut self, url: &str) -> bool {
            self.active.insert(url.to_string())
        }

        fn mark_as_processed(&mut self, url: &str) {
            self.active.remove(url);
            self.already_fetched.insert(url.to_string());
        }

        fn add_error(&mut self, err: SitemapError) {
            self.errors.push(err)
        }
    }

    fn parse_it(url: &str, body: &str) -> Result<Parsed, SitemapError> {
        parse(&body, url).map_err(|source| SitemapError::parse(url, source))
    }

    fn get_config(retailer: &RetailerCode) -> Result<SitemapConfig, SitemapError> {
        Ok(sitemap_config(retailer).ok_or_else(|| SitemapError::no_sitemap_config(*retailer))?)
    }
}
