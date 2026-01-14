# Retriever Service

| Field         | Value      |
|---------------|------------|
| Status        | Draft      |
| Owner         | @miro      |
| Created       | 2026-01-14 |
| Last updated  | 2026-01-14 |
| Related issue |            |

## Summary

The Retriever service fetches content from URLs on behalf of other services. It exposes an HTTP API that accepts fetch requests, queues them in an inbox, and processes them asynchronously.

## Responsibilities

- Accept fetch requests via HTTP API
- Authenticate callers via credentials
- Queue requests in inbox for async processing
- Worker polls inbox and fetches URL content
- Apply retry logic and rate limiting per retailer
- Write results to outbox, mark inbox message as completed

## HTTP API

### Submit Fetch Request

```
POST /fetch
Authorization: Bearer <token>
Content-Type: application/json

{
  "url": "https://example.com/page",
  "retailer": "admhr",
  "correlation_id": "uuid-123"
}
```

**Response:**
```
HTTP/1.1 200 OK
Content-Type: application/json

{
  "message_id": "msg-456",
  "status": "queued"
}
```

The service immediately returns 200 after recording the message in the inbox.

## Data Structures

### Inbox Message

```rust
pub struct InboxMessage {
    pub id: MessageId,
    pub url: Url,
    pub retailer: RetailerCode,
    pub correlation_id: String,
    pub caller_id: CallerId,
    pub status: MessageStatus,
    pub created_at: DateTime<Utc>,
    pub attempts: u32,
}

pub enum MessageStatus {
    Queued,
    Processing,
    Completed,
    Failed,
}
```

### Outbox Message

```rust
pub struct OutboxMessage {
    pub id: MessageId,
    pub inbox_message_id: MessageId,
    pub url: Url,
    pub retailer: RetailerCode,
    pub correlation_id: String,
    pub status: FetchStatus,
    pub content: Option<String>,
    pub content_type: Option<ContentType>,
    pub http_status: Option<StatusCode>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub enum FetchStatus {
    Success,
    HttpError,
    Timeout,
    NetworkError,
    RateLimited,
}
```

## Behavior

### Request Flow

```
Caller Service
       ↓
POST /fetch (with credentials)
       ↓
[Auth] → Validate credentials
       ↓
[Inbox] → Record message, return 200
```

### Worker Flow

```
[Worker] → Poll database for queued inbox messages
       ↓
[Rate Limiter] → Check retailer-specific rate limit
       ↓
[HTTP Client] → Execute GET request
       ↓
[Retry Logic] → Retry on transient failures
       ↓
[Outbox] → Create outbox message with result
       ↓
[Inbox] → Mark inbox message as completed
```

The worker continuously polls the database for new messages and processes them one by one. On successful fetch:
1. Create outbox message containing the fetched content
2. Mark inbox message as completed

On permanent failure:
1. Create outbox message with error details
2. Mark inbox message as failed

### Rate Limiting

- Per-retailer rate limits to avoid IP bans
- Configurable requests-per-second per retailer
- Queue requests when rate limit exceeded

### Retry Policy

| Error Type       | Retry | Max Attempts | Backoff        |
|------------------|-------|--------------|----------------|
| 5xx Server Error | Yes   | 3            | Exponential    |
| Timeout          | Yes   | 3            | Exponential    |
| Network Error    | Yes   | 3            | Exponential    |
| 4xx Client Error | No    | -            | -              |
| Rate Limited     | Yes   | 5            | Retry-After    |

### Timeouts

- Connection timeout: 10s
- Request timeout: 30s
- Configurable per retailer if needed

## Authentication

Callers authenticate via Bearer token in the Authorization header. Each caller has a unique `caller_id` associated with their credentials for audit and rate limiting purposes.

## Configuration

```rust
pub struct RetrieverConfig {
    // API
    pub api_port: u16,

    // Inbox
    pub inbox_poll_interval: Duration,
    pub worker_count: u32,

    // HTTP fetching
    pub default_rate_limit: RateLimit,
    pub retailer_rate_limits: HashMap<RetailerCode, RateLimit>,
    pub connection_timeout: Duration,
    pub request_timeout: Duration,
    pub max_retries: u32,
    pub user_agent: String,
}

pub struct RateLimit {
    pub requests_per_second: f64,
    pub burst_size: u32,
}
```

## Dependencies

- `axum` or `actix-web` - HTTP API framework
- `reqwest` - HTTP client for fetching
- `tokio` - Async runtime
- `sqlx` - Database (inbox persistence)
- `governor` or `leaky-bucket` - Rate limiting
- `elemental` - RetailerCode type

## Non-Goals

- HTML parsing (handled by Scraper)
- URL discovery (handled by Crawler)
- Long-term content storage (handled by caller)
- Website authentication/cookies (future scope)

## Error Handling

- API errors (auth, validation) return appropriate HTTP status codes
- Fetch errors are recorded in outbox with error details
- All errors include the original URL and correlation_id for traceability
- Transient errors are retried automatically; permanent errors (4xx) fail immediately

## Testing Strategy

- Unit tests: Mock HTTP responses, inbox/outbox operations
- Integration tests: Test API endpoints, worker processing
- Rate limit tests: Verify throttling behavior
- End-to-end tests: Submit request, verify outbox message created

## Future Considerations

- Cookie/session management for authenticated pages
- Proxy rotation for high-volume scraping
- Response caching with TTL
- Metrics and observability (request count, latency, error rate)
