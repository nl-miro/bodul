pub const COMMAND_TYPE: &str = "SourceRetailer";
pub const EVENT_TYPE_SITEMAP_RETRIEVED: &str = "SitemapRetrieved";
pub const EVENT_TYPE_RETAILER_HOMEPAGE_RETRIEVED: &str = "RetailerHomepageRetrieved";

pub mod io {
    pub use super::application::{
        RetailerHomepageRetrieved, SitemapFileRef, SitemapRetrieved, SourceRetailer,
        SourceRetailerError, SourcingMethod,
    };
    pub use super::eventing::DailySourcingFanOutSubscriber;
    pub use super::handler::SourceRetailerHandler;
}

mod application {
    use chrono::{DateTime, Utc};
    use kernel::{ApplicationCommand, ApplicationEvent};
    use retailer_guild::io::RetailerCode;
    use serde::{Deserialize, Serialize};
    use thiserror::Error;
    use uuid::Uuid;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub enum SourcingMethod {
        WebScraping,
        Sitemap,
    }

    impl SourcingMethod {
        pub fn all() -> &'static [SourcingMethod] {
            &[SourcingMethod::WebScraping, SourcingMethod::Sitemap]
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SourceRetailer {
        pub retailer_code: RetailerCode,
        pub method: SourcingMethod,
        pub requested_at: DateTime<Utc>,
    }

    impl ApplicationCommand for SourceRetailer {
        fn command_type(&self) -> &'static str {
            super::COMMAND_TYPE
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SitemapFileRef {
        pub artifact_id: Uuid,
        pub url: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SitemapRetrieved {
        pub retailer_code: RetailerCode,
        pub files: Vec<SitemapFileRef>,
        pub requested_at: DateTime<Utc>,
        pub retrieved_at: DateTime<Utc>,
    }

    impl ApplicationEvent for SitemapRetrieved {
        fn event_type(&self) -> &'static str {
            super::EVENT_TYPE_SITEMAP_RETRIEVED
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RetailerHomepageRetrieved {
        pub retailer_code: RetailerCode,
        pub artifact_id: Uuid,
        pub url: String,
        pub requested_at: DateTime<Utc>,
        pub retrieved_at: DateTime<Utc>,
    }

    impl ApplicationEvent for RetailerHomepageRetrieved {
        fn event_type(&self) -> &'static str {
            super::EVENT_TYPE_RETAILER_HOMEPAGE_RETRIEVED
        }
    }

    #[derive(Debug, Error)]
    pub enum SourceRetailerError {
        #[error("http error fetching {url}: {source}")]
        Http {
            url: String,
            #[source]
            source: reqwest::Error,
        },
        #[error("http status {status} fetching {url}")]
        BadStatus { url: String, status: u16 },
        #[error("storage error: {0}")]
        Storage(String),
        #[error("no sitemap urls discovered for retailer {0}")]
        NoSitemapDiscovered(String),
    }
}

mod handler {
    use super::application::{
        RetailerHomepageRetrieved, SitemapFileRef, SitemapRetrieved, SourceRetailer,
        SourceRetailerError, SourcingMethod,
    };
    use crate::app_event::io::RetailerSourcingEvent;
    use chrono::Utc;
    use diesel::RunQueryDsl;
    use diesel::sql_query;
    use diesel::sql_types::{Integer, Text, Uuid as SqlUuid};
    use kernel::io::{CommandError, CommandHandlerPort, DbPool};
    use reqwest::blocking::Client;
    use retailer_guild::io::RetailerCode;
    use std::mem::ManuallyDrop;
    use std::time::Duration;
    use uuid::Uuid;

    const HTTP_TIMEOUT: Duration = Duration::from_secs(30);
    const USER_AGENT: &str =
        "Mozilla/5.0 (compatible; BodulRetailerSourcing/0.1; +https://bodul.example)";
    const MAX_SITEMAP_DEPTH: usize = 4;

    pub struct SourceRetailerHandler {
        pool: DbPool,
        // ManuallyDrop so we can drop the client on a dedicated thread — its
        // inner tokio runtime panics if dropped from another runtime's context
        // (e.g. during `#[tokio::test]` teardown).
        http: ManuallyDrop<Client>,
    }

    impl SourceRetailerHandler {
        pub fn new(pool: DbPool) -> Self {
            let http = std::thread::spawn(|| {
                Client::builder()
                    .user_agent(USER_AGENT)
                    .timeout(HTTP_TIMEOUT)
                    .build()
                    .expect("failed to build reqwest client")
            })
            .join()
            .expect("http client builder thread panicked");
            Self {
                pool,
                http: ManuallyDrop::new(http),
            }
        }

        fn fetch(&self, url: &str) -> Result<(String, u16), SourceRetailerError> {
            let resp = self
                .http
                .get(url)
                .send()
                .map_err(|source| SourceRetailerError::Http {
                    url: url.to_string(),
                    source,
                })?;
            let status = resp.status();
            if !status.is_success() {
                return Err(SourceRetailerError::BadStatus {
                    url: url.to_string(),
                    status: status.as_u16(),
                });
            }
            let body = resp.text().map_err(|source| SourceRetailerError::Http {
                url: url.to_string(),
                source,
            })?;
            Ok((body, status.as_u16()))
        }

        fn store_homepage(
            &self,
            retailer_code: &RetailerCode,
            url: &str,
            body: &str,
            status_code: u16,
        ) -> Result<Uuid, SourceRetailerError> {
            let id = Uuid::now_v7();
            let mut conn = self
                .pool
                .get()
                .map_err(|e| SourceRetailerError::Storage(e.to_string()))?;
            sql_query(
                "INSERT INTO retailer_homepages (id, retailer_code, url, content, status_code) \
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind::<SqlUuid, _>(id)
            .bind::<Text, _>(retailer_code.as_string())
            .bind::<Text, _>(url)
            .bind::<Text, _>(body)
            .bind::<Integer, _>(status_code as i32)
            .execute(&mut conn)
            .map_err(|e| SourceRetailerError::Storage(e.to_string()))?;
            Ok(id)
        }

        fn store_sitemap_file(
            &self,
            retailer_code: &RetailerCode,
            url: &str,
            body: &str,
            status_code: u16,
        ) -> Result<Uuid, SourceRetailerError> {
            let id = Uuid::now_v7();
            let mut conn = self
                .pool
                .get()
                .map_err(|e| SourceRetailerError::Storage(e.to_string()))?;
            sql_query(
                "INSERT INTO retailer_sitemap_files (id, retailer_code, url, content, status_code) \
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind::<SqlUuid, _>(id)
            .bind::<Text, _>(retailer_code.as_string())
            .bind::<Text, _>(url)
            .bind::<Text, _>(body)
            .bind::<Integer, _>(status_code as i32)
            .execute(&mut conn)
            .map_err(|e| SourceRetailerError::Storage(e.to_string()))?;
            Ok(id)
        }

        fn discover_sitemap_urls(
            &self,
            retailer_code: &RetailerCode,
        ) -> Result<Vec<String>, SourceRetailerError> {
            let homepage = retailer_code
                .homepage_url()
                .trim_end_matches('/')
                .to_string();
            let robots_url = format!("{homepage}/robots.txt");
            if let Ok((body, _)) = self.fetch(&robots_url) {
                let from_robots: Vec<String> = body
                    .lines()
                    .filter_map(|line| {
                        let line = line.trim();
                        let lower = line.to_ascii_lowercase();
                        lower
                            .strip_prefix("sitemap:")
                            .map(|_| line[line.find(':').unwrap() + 1..].trim().to_string())
                    })
                    .filter(|u| !u.is_empty())
                    .collect();
                if !from_robots.is_empty() {
                    return Ok(from_robots);
                }
            }
            Ok(vec![format!("{homepage}/sitemap.xml")])
        }

        fn extract_locs(xml: &str) -> Vec<String> {
            let mut out = Vec::new();
            let mut rest = xml;
            while let Some(start) = rest.find("<loc>") {
                let after = &rest[start + 5..];
                let Some(end) = after.find("</loc>") else {
                    break;
                };
                out.push(after[..end].trim().to_string());
                rest = &after[end + 6..];
            }
            out
        }

        fn is_sitemap_index(xml: &str) -> bool {
            xml.contains("<sitemapindex")
        }

        fn process_sitemap_method(
            &self,
            cmd: &SourceRetailer,
        ) -> Result<SitemapRetrieved, SourceRetailerError> {
            let seeds = self.discover_sitemap_urls(&cmd.retailer_code)?;
            if seeds.is_empty() {
                return Err(SourceRetailerError::NoSitemapDiscovered(
                    cmd.retailer_code.as_string().to_string(),
                ));
            }

            let mut files: Vec<SitemapFileRef> = Vec::new();
            let mut queue: Vec<(String, usize)> = seeds.into_iter().map(|u| (u, 0usize)).collect();

            while let Some((url, depth)) = queue.pop() {
                let (body, status) = self.fetch(&url)?;
                let is_index = Self::is_sitemap_index(&body);
                let nested_locs = if is_index {
                    Self::extract_locs(&body)
                } else {
                    Vec::new()
                };

                let id = self.store_sitemap_file(&cmd.retailer_code, &url, &body, status)?;
                files.push(SitemapFileRef {
                    artifact_id: id,
                    url: url.clone(),
                });

                if is_index && depth < MAX_SITEMAP_DEPTH {
                    for nested in nested_locs {
                        queue.push((nested, depth + 1));
                    }
                }
            }

            Ok(SitemapRetrieved {
                retailer_code: cmd.retailer_code.clone(),
                files,
                requested_at: cmd.requested_at,
                retrieved_at: Utc::now(),
            })
        }

        fn process_web_scraping_method(
            &self,
            cmd: &SourceRetailer,
        ) -> Result<RetailerHomepageRetrieved, SourceRetailerError> {
            let url = cmd.retailer_code.homepage_url().to_string();
            let (body, status) = self.fetch(&url)?;
            let id = self.store_homepage(&cmd.retailer_code, &url, &body, status)?;
            Ok(RetailerHomepageRetrieved {
                retailer_code: cmd.retailer_code.clone(),
                artifact_id: id,
                url,
                requested_at: cmd.requested_at,
                retrieved_at: Utc::now(),
            })
        }
    }

    impl Drop for SourceRetailerHandler {
        fn drop(&mut self) {
            // SAFETY: `self.http` is only ever read via `&self` and is not
            // touched again after this take.
            let client = unsafe { ManuallyDrop::take(&mut self.http) };
            // Hand the client off to a fresh thread so its inner tokio runtime
            // doesn't try to shut down inside an outer async runtime context.
            std::thread::spawn(move || drop(client));
        }
    }

    impl CommandHandlerPort<SourceRetailer, RetailerSourcingEvent> for SourceRetailerHandler {
        fn execute(&self, cmd: SourceRetailer) -> Result<Vec<RetailerSourcingEvent>, CommandError> {
            match cmd.method {
                SourcingMethod::Sitemap => {
                    let event = self
                        .process_sitemap_method(&cmd)
                        .map_err(|e| CommandError::HandlerExecution(e.to_string()))?;
                    Ok(vec![RetailerSourcingEvent::SitemapRetrieved(event)])
                }
                SourcingMethod::WebScraping => {
                    let event = self
                        .process_web_scraping_method(&cmd)
                        .map_err(|e| CommandError::HandlerExecution(e.to_string()))?;
                    Ok(vec![RetailerSourcingEvent::RetailerHomepageRetrieved(
                        event,
                    )])
                }
            }
        }
    }
}

mod eventing {
    use super::application::{SourceRetailer, SourcingMethod};
    use crate::app_event::io::RetailerSourcingEvent;
    use kernel::io::{CommandGateway, NewCommand, NewCommandEnvelope, NewCommandMetadata};
    use kernel::{ApplicationCommand, EventError, EventSubscriberPort, NewEventEnvelope};
    use std::sync::Arc;
    use uuid::Uuid;

    fn dispatch_command<C: ApplicationCommand>(
        gateway: &CommandGateway,
        command: &C,
        correlation_id: Option<Uuid>,
        causation_event_id: Uuid,
        source: &str,
    ) -> Result<(), EventError> {
        let payload = serde_json::to_string(command)
            .map_err(|e| EventError::SubscriberExecution(e.to_string()))?;
        let envelope = NewCommandEnvelope {
            command: NewCommand {
                command_type: command.command_type().to_string(),
                payload,
            },
            metadata: Some(NewCommandMetadata {
                command_id: Uuid::now_v7(),
                correlation_id,
                causation_id: Some(causation_event_id),
                source: Some(source.to_string()),
            }),
        };
        gateway
            .dispatch(envelope)
            .map_err(|e| EventError::SubscriberExecution(e.to_string()))
    }

    pub struct DailySourcingFanOutSubscriber {
        command_gateway: Arc<CommandGateway>,
    }

    impl DailySourcingFanOutSubscriber {
        pub fn new(command_gateway: Arc<CommandGateway>) -> Self {
            Self { command_gateway }
        }
    }

    impl EventSubscriberPort for DailySourcingFanOutSubscriber {
        fn handle(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
            let meta = envelope.metadata.as_ref().ok_or_else(|| {
                EventError::SubscriberExecution(
                    "DailySourcingRequested event is missing metadata".to_string(),
                )
            })?;
            let wrapped: RetailerSourcingEvent = serde_json::from_str(&envelope.payload)
                .map_err(|e| EventError::SubscriberExecution(e.to_string()))?;
            let RetailerSourcingEvent::DailySourcingRequested(event) = wrapped else {
                return Ok(());
            };

            for retailer_code in &event.retailer_codes {
                for method in SourcingMethod::all() {
                    let cmd = SourceRetailer {
                        retailer_code: retailer_code.clone(),
                        method: *method,
                        requested_at: event.requested_at,
                    };
                    dispatch_command(
                        &self.command_gateway,
                        &cmd,
                        meta.correlation_id,
                        meta.event_id,
                        "event:DailySourcingRequested",
                    )?;
                }
            }
            Ok(())
        }
    }
}
