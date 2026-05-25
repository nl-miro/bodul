pub mod io {
    pub use super::application::{
        write_entry_error_trace_report,
        write_entry_error_trace_report_with_suffix, //
    };
}

mod domain {
    use serde_json::Value;
    use std::collections::{HashMap, HashSet};
    use uuid::Uuid;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub enum EntryKind {
        Inbox,
        Command,
        Event,
        Outbox,
    }

    impl EntryKind {
        pub fn as_str(&self) -> &'static str {
            match self {
                Self::Inbox => "inbox",
                Self::Command => "command",
                Self::Event => "event",
                Self::Outbox => "outbox",
            }
        }
    }

    impl TryFrom<&str> for EntryKind {
        type Error = String;

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            match value {
                "inbox" => Ok(Self::Inbox),
                "command" => Ok(Self::Command),
                "event" => Ok(Self::Event),
                "outbox" => Ok(Self::Outbox),
                other => Err(format!("unknown entry kind: {other}")),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct EntryStatus {
        code: i32,
    }

    impl EntryStatus {
        pub fn new(code: i32) -> Self {
            Self { code }
        }

        pub fn code(&self) -> i32 {
            self.code
        }

        pub fn as_str(&self) -> &'static str {
            match self.code {
                0 => "received",
                2 => "reserved",
                4 => "failed",
                5 => "completed",
                7 => "dead",
                8 => "archive",
                _ => "unknown",
            }
        }

        pub fn css_class(&self) -> &'static str {
            match self.code {
                4 | 7 => "status-error",
                5 => "status-completed",
                2 => "status-reserved",
                _ => "status-neutral",
            }
        }

        pub fn is_error(&self) -> bool {
            matches!(self.code, 4 | 7)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum EntryRelation {
        CausationId,
        EventId,
        MessageId,
    }

    impl EntryRelation {
        pub fn as_str(&self) -> &'static str {
            match self {
                Self::CausationId => "causation_id",
                Self::EventId => "event_id",
                Self::MessageId => "message_id",
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct EntryKey {
        pub kind: EntryKind,
        pub id: Uuid,
    }

    impl EntryKey {
        pub fn html_id(&self) -> String {
            format!("{}-{}", self.kind.as_str(), self.id)
        }
    }

    #[derive(Debug, Clone)]
    pub struct EntryRecord {
        pub key: EntryKey,
        pub name: String,
        pub status: EntryStatus,
        pub payload: String,
        pub meta: Value,
        pub attempts: i32,
        pub reservation_id: Option<Uuid>,
        pub scheduled_at: String,
        pub received_at: String,
        pub updated_at: String,
        pub processed_at: Option<String>,
        pub last_error: Option<String>,
    }

    impl EntryRecord {
        pub fn source(&self) -> Option<&str> {
            self.meta
                .get("source")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
        }

        pub fn correlation_id(&self) -> Option<Uuid> {
            self.meta_uuid("correlation_id")
        }

        pub fn causation_id(&self) -> Option<Uuid> {
            self.meta_uuid("causation_id")
        }

        pub fn event_id(&self) -> Option<Uuid> {
            self.meta_uuid("event_id")
        }

        pub fn message_id(&self) -> Option<Uuid> {
            self.meta_uuid("message_id")
        }

        pub fn trace_back_requests(&self) -> Vec<ParentRequest> {
            match self.key.kind {
                EntryKind::Inbox => vec![ParentRequest {
                    relation: EntryRelation::MessageId,
                    id: self.key.id,
                }],
                EntryKind::Command | EntryKind::Event => self
                    .causation_id()
                    .map(|id| {
                        vec![ParentRequest {
                            relation: EntryRelation::CausationId,
                            id,
                        }]
                    })
                    .unwrap_or_default(),
                EntryKind::Outbox => self
                    .event_id()
                    .map(|id| {
                        vec![ParentRequest {
                            relation: EntryRelation::EventId,
                            id,
                        }]
                    })
                    .unwrap_or_default(),
            }
        }

        fn meta_uuid(&self, field: &str) -> Option<Uuid> {
            self.meta
                .get(field)
                .and_then(Value::as_str)
                .and_then(|value| Uuid::parse_str(value).ok())
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct ParentRequest {
        pub relation: EntryRelation,
        pub id: Uuid,
    }

    #[derive(Debug, Clone)]
    pub struct TraceNode {
        pub key: EntryKey,
        pub relation: Option<EntryRelation>,
        pub parents: Vec<TraceNode>,
        pub cycle: bool,
    }

    #[derive(Debug, Default)]
    pub struct TraceIndexes {
        direct_causation_targets: HashMap<Uuid, Vec<EntryKey>>,
        events_by_id: HashMap<Uuid, Vec<EntryKey>>,
        outbox_by_message_id: HashMap<Uuid, Vec<EntryKey>>,
    }

    impl TraceIndexes {
        pub fn build(entries: &HashMap<EntryKey, EntryRecord>) -> Self {
            let mut indexes = Self::default();

            for (key, entry) in entries {
                match entry.key.kind {
                    EntryKind::Inbox | EntryKind::Command | EntryKind::Event => {
                        indexes
                            .direct_causation_targets
                            .entry(entry.key.id)
                            .or_default()
                            .push(key.clone());
                    }
                    EntryKind::Outbox => {
                        if let Some(message_id) = entry.message_id() {
                            indexes
                                .outbox_by_message_id
                                .entry(message_id)
                                .or_default()
                                .push(key.clone());
                        }
                    }
                }

                if entry.key.kind == EntryKind::Event {
                    indexes
                        .events_by_id
                        .entry(entry.key.id)
                        .or_default()
                        .push(key.clone());
                }
            }

            indexes
        }

        pub fn trace(
            &self,
            root: &EntryKey,
            entries: &HashMap<EntryKey, EntryRecord>,
        ) -> TraceNode {
            let mut path = HashSet::new();
            self.trace_node(root, None, entries, &mut path)
        }

        fn trace_node(
            &self,
            key: &EntryKey,
            relation: Option<EntryRelation>,
            entries: &HashMap<EntryKey, EntryRecord>,
            path: &mut HashSet<EntryKey>,
        ) -> TraceNode {
            if !path.insert(key.clone()) {
                return TraceNode {
                    key: key.clone(),
                    relation,
                    parents: vec![],
                    cycle: true,
                };
            }

            let entry = entries
                .get(key)
                .expect("trace_node should only be called for known entries");

            let mut parent_keys = self.parent_keys(entry, key);
            parent_keys.sort_by(|left, right| left.1.cmp(&right.1));

            let parents = parent_keys
                .into_iter()
                .map(|(parent_relation, parent_key)| {
                    self.trace_node(&parent_key, Some(parent_relation), entries, path)
                })
                .collect();

            path.remove(key);

            TraceNode {
                key: key.clone(),
                relation,
                parents,
                cycle: false,
            }
        }

        fn parent_keys(
            &self,
            entry: &EntryRecord,
            current: &EntryKey,
        ) -> Vec<(EntryRelation, EntryKey)> {
            let mut seen = HashSet::new();
            let mut parents = Vec::new();

            for request in entry.trace_back_requests() {
                let candidates = match request.relation {
                    EntryRelation::CausationId => self
                        .direct_causation_targets
                        .get(&request.id)
                        .or_else(|| self.outbox_by_message_id.get(&request.id)),
                    EntryRelation::EventId => self.events_by_id.get(&request.id),
                    EntryRelation::MessageId => self.outbox_by_message_id.get(&request.id),
                };

                for candidate in candidates.into_iter().flatten() {
                    if candidate == current {
                        continue;
                    }

                    if seen.insert(candidate.clone()) {
                        parents.push((request.relation, candidate.clone()));
                    }
                }
            }

            parents
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::json;

        fn entry(
            kind: EntryKind,
            id: Uuid,
            name: &str,
            status: i32,
            meta: Value,
        ) -> (EntryKey, EntryRecord) {
            let key = EntryKey { kind, id };
            (
                key.clone(),
                EntryRecord {
                    key,
                    name: name.to_string(),
                    status: EntryStatus::new(status),
                    payload: "{}".to_string(),
                    meta,
                    attempts: 1,
                    reservation_id: None,
                    scheduled_at: "2026-01-01 00:00:00+00".to_string(),
                    received_at: "2026-01-01 00:00:00+00".to_string(),
                    updated_at: "2026-01-01 00:00:00+00".to_string(),
                    processed_at: Some("2026-01-01 00:00:00+00".to_string()),
                    last_error: None,
                },
            )
        }

        #[test]
        fn traces_outbox_back_to_event_and_command() {
            let command_id = Uuid::now_v7();
            let event_id = Uuid::now_v7();
            let message_id = Uuid::now_v7();

            let entries = HashMap::from([
                entry(
                    EntryKind::Command,
                    command_id,
                    "StartDailySourcing",
                    5,
                    json!({}),
                ),
                entry(
                    EntryKind::Event,
                    event_id,
                    "DailySourcingRequested",
                    5,
                    json!({
                        "causation_id": command_id.to_string(),
                    }),
                ),
                entry(
                    EntryKind::Outbox,
                    event_id,
                    "DailySourcingRequested",
                    4,
                    json!({
                        "event_id": event_id.to_string(),
                        "message_id": message_id.to_string(),
                    }),
                ),
            ]);

            let indexes = TraceIndexes::build(&entries);
            let root = EntryKey {
                kind: EntryKind::Outbox,
                id: event_id,
            };
            let trace = indexes.trace(&root, &entries);

            assert_eq!(trace.parents.len(), 1);
            assert_eq!(trace.parents[0].key.kind, EntryKind::Event);
            assert_eq!(trace.parents[0].parents.len(), 1);
            assert_eq!(trace.parents[0].parents[0].key.kind, EntryKind::Command);
            assert_eq!(trace.parents[0].parents[0].key.id, command_id);
        }

        #[test]
        fn traces_command_back_through_inbox_to_outbox_chain() {
            let root_command_id = Uuid::now_v7();
            let event_id = Uuid::now_v7();
            let message_id = Uuid::now_v7();
            let failed_command_id = Uuid::now_v7();

            let entries = HashMap::from([
                entry(
                    EntryKind::Command,
                    root_command_id,
                    "StartDailySourcing",
                    5,
                    json!({}),
                ),
                entry(
                    EntryKind::Event,
                    event_id,
                    "DailySourcingRequested",
                    5,
                    json!({
                        "causation_id": root_command_id.to_string(),
                    }),
                ),
                entry(
                    EntryKind::Outbox,
                    event_id,
                    "DailySourcingRequested",
                    5,
                    json!({
                        "event_id": event_id.to_string(),
                        "message_id": message_id.to_string(),
                    }),
                ),
                entry(EntryKind::Inbox, message_id, "retailer.inbox", 5, json!({})),
                entry(
                    EntryKind::Command,
                    failed_command_id,
                    "SourceRetailer",
                    4,
                    json!({
                        "causation_id": message_id.to_string(),
                    }),
                ),
            ]);

            let indexes = TraceIndexes::build(&entries);
            let root = EntryKey {
                kind: EntryKind::Command,
                id: failed_command_id,
            };
            let trace = indexes.trace(&root, &entries);

            assert_eq!(trace.parents.len(), 1);
            assert_eq!(trace.parents[0].key.kind, EntryKind::Inbox);
            assert_eq!(trace.parents[0].parents.len(), 1);
            assert_eq!(trace.parents[0].parents[0].key.kind, EntryKind::Outbox);
            assert_eq!(trace.parents[0].parents[0].parents.len(), 1);
            assert_eq!(
                trace.parents[0].parents[0].parents[0].key.kind,
                EntryKind::Event
            );
            assert_eq!(
                trace.parents[0].parents[0].parents[0].parents[0].key.kind,
                EntryKind::Command
            );
        }
    }
}

mod application {
    use super::domain::{EntryKey, EntryKind, EntryRecord, EntryStatus, TraceIndexes, TraceNode};
    use crate::io::DbPool;
    use diesel::prelude::*;
    use diesel::sql_query;
    use diesel::sql_types::{Integer, Nullable, Text, Uuid as SqlUuid};
    use serde_json::Value;
    use std::collections::HashMap;
    use std::fmt::Write as _;
    use std::fs;
    use std::path::PathBuf;
    use uuid::Uuid;

    const REPORT_FILE_NAME: &str = "entry-error-tracing.html";

    #[derive(Debug, QueryableByName)]
    struct RawEntryRow {
        #[diesel(sql_type = Text)]
        kind: String,
        #[diesel(sql_type = Text)]
        name: String,
        #[diesel(sql_type = SqlUuid)]
        id: Uuid,
        #[diesel(sql_type = Integer)]
        status: i32,
        #[diesel(sql_type = Text)]
        payload: String,
        #[diesel(sql_type = Text)]
        meta: String,
        #[diesel(sql_type = Integer)]
        attempts: i32,
        #[diesel(sql_type = Nullable<SqlUuid>)]
        reservation_id: Option<Uuid>,
        #[diesel(sql_type = Text)]
        scheduled_at: String,
        #[diesel(sql_type = Text)]
        received_at: String,
        #[diesel(sql_type = Text)]
        updated_at: String,
        #[diesel(sql_type = Nullable<Text>)]
        processed_at: Option<String>,
        #[diesel(sql_type = Nullable<Text>)]
        last_error: Option<String>,
    }

    pub fn write_entry_error_trace_report(
        pool: &DbPool,
    ) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        write_entry_error_trace_report_with_suffix(pool, None)
    }

    pub fn write_entry_error_trace_report_with_suffix(
        pool: &DbPool,
        suffix: Option<&str>,
    ) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let entries = load_entries(pool)?;
        let indexes = TraceIndexes::build(&entries);
        let mut roots = entries
            .values()
            .filter(|entry| entry.status.is_error())
            .map(|entry| entry.key.clone())
            .collect::<Vec<_>>();
        roots.sort();

        let traces = roots
            .iter()
            .map(|root| indexes.trace(root, &entries))
            .collect::<Vec<_>>();

        let output_path = report_path(suffix);
        let html = render_html(&entries, &traces, &output_path);
        fs::write(&output_path, html)?;

        Ok(output_path)
    }

    fn load_entries(
        pool: &DbPool,
    ) -> Result<HashMap<EntryKey, EntryRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = pool.get()?;
        let rows = sql_query(
            r#"
            SELECT *
            FROM (
                SELECT
                    'inbox'::text AS kind,
                    COALESCE(meta->>'routing_key', 'inbox') AS name,
                    id,
                    status,
                    payload,
                    meta::text AS meta,
                    attempts,
                    reservation_id,
                    scheduled_at::text AS scheduled_at,
                    received_at::text AS received_at,
                    updated_at::text AS updated_at,
                    processed_at::text AS processed_at,
                    NULL::text AS last_error
                FROM inbox_entries

                UNION ALL

                SELECT
                    'command'::text AS kind,
                    command_type AS name,
                    id,
                    status,
                    payload,
                    COALESCE(meta::text, 'null') AS meta,
                    attempts,
                    reservation_id,
                    scheduled_at::text AS scheduled_at,
                    received_at::text AS received_at,
                    updated_at::text AS updated_at,
                    processed_at::text AS processed_at,
                    NULL::text AS last_error
                FROM command_entries

                UNION ALL

                SELECT
                    'event'::text AS kind,
                    event_type AS name,
                    id,
                    status,
                    payload,
                    COALESCE(meta::text, 'null') AS meta,
                    attempts,
                    reservation_id,
                    scheduled_at::text AS scheduled_at,
                    received_at::text AS received_at,
                    updated_at::text AS updated_at,
                    processed_at::text AS processed_at,
                    NULL::text AS last_error
                FROM event_entries

                UNION ALL

                SELECT
                    'outbox'::text AS kind,
                    COALESCE(meta->>'event_type', meta->>'routing_key', 'outbox') AS name,
                    id,
                    status,
                    payload,
                    meta::text AS meta,
                    attempts,
                    reservation_id,
                    scheduled_at::text AS scheduled_at,
                    received_at::text AS received_at,
                    updated_at::text AS updated_at,
                    processed_at::text AS processed_at,
                    last_error
                FROM outbox_entries
            ) entries
            ORDER BY received_at ASC, kind ASC, name ASC
            "#,
        )
        .load::<RawEntryRow>(&mut conn)?;

        rows.into_iter()
            .map(|row| {
                let kind = EntryKind::try_from(row.kind.as_str())?;
                let key = EntryKey { kind, id: row.id };
                let meta = serde_json::from_str::<Value>(&row.meta).map_err(|err| {
                    format!(
                        "invalid meta json for {} {}: {err}",
                        key.kind.as_str(),
                        key.id
                    )
                })?;

                Ok((
                    key.clone(),
                    EntryRecord {
                        key,
                        name: row.name,
                        status: EntryStatus::new(row.status),
                        payload: row.payload,
                        meta,
                        attempts: row.attempts,
                        reservation_id: row.reservation_id,
                        scheduled_at: row.scheduled_at,
                        received_at: row.received_at,
                        updated_at: row.updated_at,
                        processed_at: row.processed_at,
                        last_error: row.last_error,
                    },
                ))
            })
            .collect()
    }

    fn render_html(
        entries: &HashMap<EntryKey, EntryRecord>,
        traces: &[TraceNode],
        output_path: &PathBuf,
    ) -> String {
        let generated_at = chrono::Utc::now().to_rfc3339();
        let error_count = traces.len();
        let total_entries = entries.len();

        let mut html = String::new();
        html.push_str("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">");
        html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
        html.push_str("<title>Retailer Sourcing Entry Error Trace</title><style>");
        html.push_str(
            "body{font-family:system-ui,-apple-system,BlinkMacSystemFont,\"Segoe UI\",sans-serif;background:#111827;color:#e5e7eb;margin:0;padding:2rem;line-height:1.5;}\
             a{color:#93c5fd;}\
             h1,h2,h3{margin-top:0;}\
             .summary{background:#1f2937;border:1px solid #374151;border-radius:12px;padding:1rem 1.25rem;margin-bottom:1.5rem;}\
             .summary code{color:#d1fae5;}\
             .trace{margin-bottom:1.5rem;}\
             .trace ul{list-style:none;margin:0;padding-left:1.5rem;border-left:1px solid #374151;}\
             .trace > ul{padding-left:0;border-left:none;}\
             .trace li{margin:0.75rem 0;}\
             .entry-card{background:#1f2937;border:1px solid #374151;border-left-width:6px;border-radius:12px;padding:0.9rem 1rem;}\
             .status-error{border-left-color:#dc2626;color:#fecaca;}\
             .status-completed{border-left-color:#16a34a;color:#bbf7d0;}\
             .status-reserved{border-left-color:#f59e0b;color:#fde68a;}\
             .status-neutral{border-left-color:#6b7280;color:#e5e7eb;}\
             .entry-header{display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;margin-bottom:0.5rem;}\
             .entry-kind{font-weight:700;text-transform:uppercase;letter-spacing:0.05em;font-size:0.8rem;}\
             .status-pill{display:inline-block;padding:0.2rem 0.55rem;border-radius:999px;background:#0f172a;border:1px solid currentColor;font-size:0.85rem;}\
             .relation{display:inline-block;margin-bottom:0.35rem;font-size:0.82rem;color:#9ca3af;}\
             .entry-table{width:100%;border-collapse:collapse;table-layout:fixed;margin-bottom:0.75rem;}\
             .entry-table th,.entry-table td{padding:0.45rem 0.55rem;border:1px solid #374151;vertical-align:top;word-break:break-word;}\
             .entry-table th{font-size:0.78rem;color:#9ca3af;font-weight:600;text-align:left;background:#111827;}\
             .entry-table td{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;background:#0f172a;}\
             details{margin-top:0.5rem;background:#0f172a;border:1px solid #374151;border-radius:8px;padding:0.5rem 0.75rem;}\
             pre{white-space:pre-wrap;word-break:break-word;margin:0.5rem 0 0;}\
             .empty{padding:1rem 1.25rem;background:#1f2937;border:1px solid #374151;border-radius:12px;}\
             .cycle{margin-top:0.5rem;color:#fca5a5;font-size:0.9rem;}",
        );
        html.push_str("</style></head><body>");

        html.push_str("<div class=\"summary\">");
        html.push_str("<h1>Retailer Sourcing Entry Error Trace</h1>");
        let _ = write!(
            html,
            "<p>Generated at <code>{}</code>.</p><p>Scanned <strong>{}</strong> entries and found <strong>{}</strong> errors. Report path: <code>{}</code>.</p>",
            escape_html(&generated_at),
            total_entries,
            error_count,
            escape_html(&output_path.display().to_string()),
        );
        html.push_str("</div>");

        if traces.is_empty() {
            html.push_str(
                "<div class=\"empty\"><h2>No error entries found</h2><p>No inbox, command, event, or outbox entries are currently in <code>failed</code> or <code>dead</code> state.</p></div>",
            );
        } else {
            html.push_str("<div class=\"summary\"><h2>Error roots</h2><ul>");
            for trace in traces {
                let entry = &entries[&trace.key];
                let _ = write!(
                    html,
                    "<li><a href=\"#{}\">{} {} ({})</a></li>",
                    escape_html(&trace.key.html_id()),
                    escape_html(trace.key.kind.as_str()),
                    escape_html(&entry.name),
                    escape_html(entry.status.as_str()),
                );
            }
            html.push_str("</ul></div>");

            for trace in traces {
                let entry = &entries[&trace.key];
                let _ = write!(
                    html,
                    "<section class=\"trace\" id=\"{}\"><h2>{} {} — {}</h2><ul>",
                    escape_html(&trace.key.html_id()),
                    escape_html(trace.key.kind.as_str()),
                    escape_html(&entry.name),
                    escape_html(entry.status.as_str()),
                );
                render_trace_node(&mut html, trace, entries);
                html.push_str("</ul></section>");
            }
        }

        html.push_str("</body></html>");
        html
    }

    fn render_trace_node(
        html: &mut String,
        node: &TraceNode,
        entries: &HashMap<EntryKey, EntryRecord>,
    ) {
        let entry = &entries[&node.key];
        let _ = write!(html, "<li>");

        if let Some(relation) = node.relation {
            let _ = write!(
                html,
                "<div class=\"relation\">trace via {}</div>",
                escape_html(relation.as_str()),
            );
        }

        let _ = write!(
            html,
            "<article class=\"entry-card {}\">",
            entry.status.css_class(),
        );
        let _ = write!(
            html,
            "<div class=\"entry-header\"><span class=\"entry-kind\">{}</span><strong>{}</strong><span class=\"status-pill\">{} ({})</span></div>",
            escape_html(node.key.kind.as_str()),
            escape_html(&entry.name),
            escape_html(entry.status.as_str()),
            entry.status.code(),
        );

        let mut metadata = vec![
            ("id", entry.key.id.to_string()),
            ("attempts", entry.attempts.to_string()),
            ("scheduled_at", entry.scheduled_at.clone()),
            ("received_at", entry.received_at.clone()),
            ("updated_at", entry.updated_at.clone()),
        ];
        push_optional_metadata(&mut metadata, "processed_at", entry.processed_at.clone());
        push_optional_metadata(
            &mut metadata,
            "reservation_id",
            entry.reservation_id.map(|id| id.to_string()),
        );
        push_optional_metadata(
            &mut metadata,
            "correlation_id",
            entry.correlation_id().map(|id| id.to_string()),
        );
        push_optional_metadata(
            &mut metadata,
            "causation_id",
            entry.causation_id().map(|id| id.to_string()),
        );
        push_optional_metadata(
            &mut metadata,
            "event_id",
            entry.event_id().map(|id| id.to_string()),
        );
        push_optional_metadata(
            &mut metadata,
            "message_id",
            entry.message_id().map(|id| id.to_string()),
        );
        push_optional_metadata(
            &mut metadata,
            "source",
            entry.source().map(ToString::to_string),
        );
        push_optional_metadata(&mut metadata, "last_error", entry.last_error.clone());
        render_metadata_table(html, &metadata);

        let open_details = entry.status.is_error()
            || matches!(
                node.relation,
                Some(super::domain::EntryRelation::CausationId)
            );

        render_json_details(html, "payload", &entry.payload, open_details);
        render_value_details(html, "meta", &entry.meta, open_details);

        if node.cycle {
            html.push_str("<div class=\"cycle\">Cycle detected while tracing this branch.</div>");
        }

        html.push_str("</article>");

        if !node.parents.is_empty() {
            html.push_str("<ul>");
            for parent in &node.parents {
                render_trace_node(html, parent, entries);
            }
            html.push_str("</ul>");
        }

        html.push_str("</li>");
    }

    fn push_optional_metadata(
        metadata: &mut Vec<(&'static str, String)>,
        label: &'static str,
        value: Option<String>,
    ) {
        if let Some(value) = value.filter(|value| !value.is_empty()) {
            metadata.push((label, value));
        }
    }

    fn render_metadata_table(html: &mut String, metadata: &[(&str, String)]) {
        html.push_str("<table class=\"entry-table\"><tr>");
        for (label, _) in metadata {
            let _ = write!(html, "<th>{}</th>", escape_html(label));
        }
        html.push_str("</tr><tr>");
        for (_, value) in metadata {
            let _ = write!(html, "<td>{}</td>", escape_html(value));
        }
        html.push_str("</tr></table>");
    }

    fn render_json_details(html: &mut String, label: &str, raw: &str, is_open: bool) {
        let formatted = serde_json::from_str::<Value>(raw)
            .ok()
            .and_then(|value| serde_json::to_string_pretty(&value).ok())
            .unwrap_or_else(|| raw.to_string());
        let open_attr = if is_open { " open" } else { "" };
        let _ = write!(
            html,
            "<details{}><summary>{}</summary><pre>{}</pre></details>",
            open_attr,
            escape_html(label),
            escape_html(&formatted),
        );
    }

    fn render_value_details(html: &mut String, label: &str, value: &Value, is_open: bool) {
        let formatted = serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
        let open_attr = if is_open { " open" } else { "" };
        let _ = write!(
            html,
            "<details{}><summary>{}</summary><pre>{}</pre></details>",
            open_attr,
            escape_html(label),
            escape_html(&formatted),
        );
    }

    fn report_path(suffix: Option<&str>) -> PathBuf {
        let file_name = match suffix.filter(|suffix| !suffix.is_empty()) {
            Some(suffix) => format!("entry-error-tracing_{suffix}.html"),
            None => REPORT_FILE_NAME.to_string(),
        };

        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(file_name)
    }

    fn escape_html(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
}
