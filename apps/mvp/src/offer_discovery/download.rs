pub const DOWNLOAD_OFFER_PAGE_COMMAND: &str = "DownloadOfferPage";
pub const OFFER_PAGE_WAS_DOWNLOADED_EVENT: &str = "OfferPageWasDownloaded";
pub const OFFER_PAGE_DOWNLOAD_SKIPPED_EVENT: &str = "OfferPageDownloadSkipped";

/// How recently an offer URL must have been fetched to skip re-downloading it.
const REDOWNLOAD_WINDOW_HOURS: i64 = 6;

pub mod io {
    pub use super::handler::DownloadOfferPageHandler;
    pub use super::models::{DownloadOfferPage, OfferPageDownloadSkipped, OfferPageWasDownloaded};
    pub use super::repository::{OfferRepository, RawOfferRepository};
    pub use super::{DOWNLOAD_OFFER_PAGE_COMMAND, OFFER_PAGE_DOWNLOAD_SKIPPED_EVENT, OFFER_PAGE_WAS_DOWNLOADED_EVENT};
}

mod models {
    use kernel::{ApplicationCommand, ApplicationEvent};
    use serde::{Deserialize, Serialize};
    use bodul_shared::retailer::RetailerCode;
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DownloadOfferPage {
        pub grouped_content_id: Uuid,
        pub retailer_code: RetailerCode,
        pub url: String,
    }

    impl ApplicationCommand for DownloadOfferPage {
        fn command_type(&self) -> &'static str {
            super::DOWNLOAD_OFFER_PAGE_COMMAND
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OfferPageWasDownloaded {
        pub offer_id: Uuid,
        pub grouped_content_id: Uuid,
        pub retailer_code: RetailerCode,
        pub url: String,
    }

    impl ApplicationEvent for OfferPageWasDownloaded {
        fn event_type(&self) -> &'static str {
            super::OFFER_PAGE_WAS_DOWNLOADED_EVENT
        }
    }

    /// Emitted when a `DownloadOfferPage` is skipped because the URL was fetched too recently;
    /// carries the id of the already-stored `raw_offer`.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OfferPageDownloadSkipped {
        pub offer_id: Uuid,
        pub grouped_content_id: Uuid,
        pub retailer_code: RetailerCode,
        pub url: String,
        pub raw_offer_id: Uuid,
    }

    impl ApplicationEvent for OfferPageDownloadSkipped {
        fn event_type(&self) -> &'static str {
            super::OFFER_PAGE_DOWNLOAD_SKIPPED_EVENT
        }
    }
}

pub mod repository {
    use crate::RepositoryError;
    use crate::database::DbPool;
    use crate::offer_discovery::model::{NewOfferRecord, NewRawOfferRecord};
    use crate::schema::{offers, raw_offers};
    use chrono::{DateTime, Utc};
    use diesel::prelude::*;
    use uuid::Uuid;

    pub struct OfferRepository {
        pool: DbPool,
    }

    impl OfferRepository {
        pub fn new(pool: DbPool) -> Self {
            Self { pool }
        }

        /// Insert the offer (or reset an existing `(retailer_code, url)` row) and return its id.
        pub fn store(&self, record: NewOfferRecord) -> Result<Uuid, RepositoryError> {
            let mut connection = self.pool.get()?;

            let id = diesel::insert_into(offers::table)
                .values(&record)
                .on_conflict((offers::retailer_code, offers::url))
                .do_update()
                .set(offers::status.eq(record.status))
                .returning(offers::id)
                .get_result(&mut *connection)?;

            Ok(id)
        }

        pub fn mark_downloaded(&self, offer_id: Uuid) -> Result<(), RepositoryError> {
            let mut connection = self.pool.get()?;

            diesel::update(offers::table.find(offer_id))
                .set((offers::status.eq("downloaded"), offers::notes.eq(None::<String>)))
                .execute(&mut *connection)?;

            Ok(())
        }

        pub fn mark_failed(&self, offer_id: Uuid, notes: &str) -> Result<(), RepositoryError> {
            let mut connection = self.pool.get()?;

            diesel::update(offers::table.find(offer_id))
                .set((offers::status.eq("failed"), offers::notes.eq(notes)))
                .execute(&mut *connection)?;

            Ok(())
        }
    }

    pub struct RawOfferRepository {
        pool: DbPool,
    }

    impl RawOfferRepository {
        pub fn new(pool: DbPool) -> Self {
            Self { pool }
        }

        /// Return the id of the most recent `raw_offer` for `url` fetched at/after `since`, if any.
        pub fn find_recent_by_url(&self, url: &str, since: DateTime<Utc>) -> Result<Option<Uuid>, RepositoryError> {
            let mut connection = self.pool.get()?;

            let raw_offer_id = raw_offers::table
                .filter(raw_offers::url.eq(url))
                .filter(raw_offers::fetched_at.ge(since))
                .order(raw_offers::fetched_at.desc())
                .select(raw_offers::id)
                .first::<Uuid>(&mut *connection)
                .optional()?;

            Ok(raw_offer_id)
        }

        /// Store the downloaded page body, replacing any previously stored body for the offer.
        pub fn store(&self, record: NewRawOfferRecord) -> Result<(), RepositoryError> {
            let mut connection = self.pool.get()?;

            diesel::insert_into(raw_offers::table)
                .values(&record)
                .on_conflict(raw_offers::offer_id)
                .do_update()
                .set((
                    raw_offers::body.eq(&record.body),
                    raw_offers::body_size.eq(record.body_size),
                    raw_offers::fetched_at.eq(diesel::dsl::now),
                ))
                .execute(&mut *connection)?;

            Ok(())
        }
    }
}

mod handler {
    use super::models::{DownloadOfferPage, OfferPageDownloadSkipped, OfferPageWasDownloaded};
    use super::repository::{OfferRepository, RawOfferRepository};
    use crate::IntoCommandError;
    use crate::RepositoryError;
    use crate::assembly::io::MvpEvent;
    use crate::offer_discovery::model::{NewOfferRecord, NewRawOfferRecord};
    use crate::retailer_data_ingestion::Client;
    use kernel::io::{CommandError, CommandHandlerPort};
    use uuid::Uuid;

    pub struct DownloadOfferPageHandler {
        offer_repo: OfferRepository,
        raw_offer_repo: RawOfferRepository,
    }

    impl DownloadOfferPageHandler {
        pub fn new(offer_repo: OfferRepository, raw_offer_repo: RawOfferRepository) -> Self {
            Self {
                offer_repo,
                raw_offer_repo,
            }
        }

        fn build_offer_record(command: &DownloadOfferPage, status: &'static str) -> NewOfferRecord {
            NewOfferRecord {
                id: Uuid::now_v7(),
                grouped_content_id: command.grouped_content_id,
                retailer_code: command.retailer_code.slug().to_string(),
                url: command.url.clone(),
                status,
                notes: None,
                discovered_at: chrono::Utc::now(),
            }
        }

        fn build_raw_offer_record(offer_id: Uuid, url: &str, body: String) -> Result<NewRawOfferRecord, CommandError> {
            let body_size = i32::try_from(body.len())
                .map_err(|_| RepositoryError::Unexpected("offer page body is too large for storage".to_string()))
                .storage_err()?;

            Ok(NewRawOfferRecord {
                id: Uuid::now_v7(),
                offer_id,
                url: url.to_string(),
                body,
                body_size,
            })
        }
    }

    impl CommandHandlerPort<DownloadOfferPage, MvpEvent> for DownloadOfferPageHandler {
        fn execute(&self, command: DownloadOfferPage) -> Result<Vec<MvpEvent>, CommandError> {
            // Skip re-downloading a URL fetched within the re-download window; reuse the existing raw_offer.
            let cutoff = chrono::Utc::now() - chrono::Duration::hours(super::REDOWNLOAD_WINDOW_HOURS);

            if let Some(raw_offer_id) = self
                .raw_offer_repo
                .find_recent_by_url(&command.url, cutoff)
                .storage_err()?
            {
                let offer_id = self
                    .offer_repo
                    .store(Self::build_offer_record(&command, "skipped6hlimit"))
                    .storage_err()?;

                let event = OfferPageDownloadSkipped {
                    offer_id,
                    grouped_content_id: command.grouped_content_id,
                    retailer_code: command.retailer_code,
                    url: command.url,
                    raw_offer_id,
                };
                return Ok(vec![MvpEvent::OfferPageDownloadSkipped(event)]);
            }

            let record = Self::build_offer_record(&command, "discovered");
            let offer_id = self.offer_repo.store(record).storage_err()?;

            let body = match Self::download_offer_body(&command) {
                Ok(body) => body,
                Err(error) => {
                    // Record the failure on the offer and stop — the command is done, not retried.
                    self.offer_repo
                        .mark_failed(offer_id, &error.to_string())
                        .storage_err()?;
                    return Ok(vec![]);
                }
            };

            let raw_record = Self::build_raw_offer_record(offer_id, &command.url, body)?;
            self.raw_offer_repo.store(raw_record).storage_err()?;
            self.offer_repo.mark_downloaded(offer_id).storage_err()?;

            let event = OfferPageWasDownloaded {
                offer_id,
                grouped_content_id: command.grouped_content_id,
                retailer_code: command.retailer_code,
                url: command.url,
            };
            Ok(vec![MvpEvent::OfferPageWasDownloaded(event)])
        }
    }

    impl DownloadOfferPageHandler {
        fn download_offer_body(
            command: &DownloadOfferPage,
        ) -> Result<String, crate::retailer_data_ingestion::FetchError> {
            Client::get_for_retailer(command.retailer_code, &command.url)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use bodul_shared::retailer::RetailerCode;
        use uuid::Uuid;

        #[test]
        fn builds_discovered_offer_record_from_command() {
            let grouped_content_id = Uuid::now_v7();
            let command = DownloadOfferPage {
                grouped_content_id,
                retailer_code: RetailerCode::AnkerCom,
                url: "https://www.anker.com/products/a1".to_string(),
            };

            let record = DownloadOfferPageHandler::build_offer_record(&command, "discovered");

            assert_eq!(record.grouped_content_id, grouped_content_id);
            assert_eq!(record.retailer_code, RetailerCode::AnkerCom.slug().to_string());
            assert_eq!(record.url, "https://www.anker.com/products/a1");
            assert_eq!(record.status, "discovered");
        }

        #[test]
        fn builds_skipped_offer_record_with_status() {
            let command = DownloadOfferPage {
                grouped_content_id: Uuid::now_v7(),
                retailer_code: RetailerCode::AnkerCom,
                url: "https://www.anker.com/products/a1".to_string(),
            };

            let record = DownloadOfferPageHandler::build_offer_record(&command, "skipped6hlimit");

            assert_eq!(record.status, "skipped6hlimit");
        }

        #[test]
        fn builds_raw_offer_record_with_body_size() {
            let offer_id = Uuid::now_v7();
            let body = "<html>offer</html>".to_string();

            let record =
                DownloadOfferPageHandler::build_raw_offer_record(offer_id, "https://example.com/p/1", body.clone())
                    .unwrap();

            assert_eq!(record.offer_id, offer_id);
            assert_eq!(record.url, "https://example.com/p/1");
            assert_eq!(record.body, body);
            assert_eq!(record.body_size, body.len() as i32);
        }
    }
}
