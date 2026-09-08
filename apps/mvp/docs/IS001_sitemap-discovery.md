# Sitemap Discovery — Implementation Plan

| Field   | Value                                 |
| ------- | ------------------------------------- |
| Status  | Draft                                 |
| Version | 0.1.0                                 |
| Scope   | `apps/mvp` module `sitemap_discovery` |
| Roadmap | Stage B                               |

The first implementation plan for `sitemap_discovery`. It fetches a retailer's
root sitemap, follows it to every sub-sitemap, and returns the whole tree as one
data structure. Fetching is delegated to `retailer_data_ingestion`'s client.

This is **discovery only**: fetch and parse, then hand back data. Storing the
result (Stage B "store") and classifying URLs as product/collection/catalog
(Stage C) belong to the caller, not here.

---

## 1. Public API

One entry point:

```rust
pub fn fetch_sitemap(retailer: RetailerCode) -> Result<SitemapDocument, SitemapError>;
```

- Input is just the `RetailerCode` (from `lib/bodul_shared`).
- The HTTP client is used internally — callers don't pass it in.
- Returns the root `SitemapDocument` (§2) with its `children` already fetched and
  parsed, so the caller can query the whole tree as one node. Or a typed
  `SitemapError`.

---

## 2. Data model

A parsed sitemap is a **composite tree**: one `SitemapDocument` holds both its
own page entries (`urls`) and any child documents (`children`). A sitemap index
and its sub-sitemaps therefore form a single tree that behaves like one node —
the accessors recurse through `children`. These types are implemented in [`src/sitemap_discovery/sitemap.rs`](../src/sitemap_discovery/sitemap.rs).

```rust
/// A parsed sitemap as a composite tree.
pub struct SitemapDocument {
    pub location: Option<String>,            // this document's own URL
    pub last_modified: Option<DateTime<Utc>>, // <lastmod> when listed in a parent
    pub urls: Vec<SitemapUrl>,               // entries declared in this document
    pub children: Vec<SitemapDocument>,      // sub-documents (from a <sitemapindex>)
}

impl SitemapDocument {
    pub fn kind(&self) -> SitemapKind;                       // from location; None -> Other
    pub fn all_urls(&self) -> impl Iterator<Item = &SitemapUrl>; // this doc + all descendants
    pub fn urls_of_kind(&self, kind: SitemapKind) -> Vec<&SitemapUrl>; // recursive, filtered
}

/// Sitemap kind inferred from the URL location (keyword match on the filename).
pub enum SitemapKind { Product, Collection, Catalog, Other }

/// One `<url>` entry.
pub struct SitemapUrl {
    pub location: String,
    pub last_modified: Option<DateTime<Utc>>,  // <lastmod>
    pub change_frequency: Option<ChangeFrequency>, // <changefreq>
    pub priority: Option<f32>,                 // <priority>, in [0.0, 1.0]
    pub images: Vec<SitemapImage>,             // Shopify image extension
}

/// Image metadata nested under a URL entry.
pub struct SitemapImage {
    pub location: String,   // <image:loc>
    pub title: Option<String>,   // <image:title>
    pub caption: Option<String>, // <image:caption>
}

/// Values accepted by `<changefreq>`.
pub enum ChangeFrequency { Always, Hourly, Daily, Weekly, Monthly, Yearly, Never }
```

Notes:

- **Composite, recursive.** An index becomes a `SitemapDocument` whose `children`
  are the fetched sub-documents; a flat sitemap is a `SitemapDocument` with `urls`
  and no `children`. `all_urls` and `urls_of_kind` descend the whole tree, so the
  root answers as if it were one document.
- `kind()` derives from `location` (keyword match on the filename — `product`,
  `catalog`, `collection`, else `Other`); a document with no location is `Other`.
- `ChangeFrequency` parses and formats the standard `<changefreq>` values
  (`FromStr` / `Display`); an unrecognised value yields `ParseChangeFrequencyError`.
- `last_modified` uses `chrono::DateTime<Utc>`.
- `images` keeps the image data Shopify ships in product sitemaps, so the result
  carries *all* the information the sitemaps contain.

---

## 3. How it works

`fetch_sitemap` runs four steps:

1. **Resolve the root URL.** Map the `RetailerCode` to its root sitemap URL. In
   Phase 0 this is hardcoded (Minisforum is a Shopify store, so the root is
   `…/sitemap.xml`). Discovering it via `robots.txt` is deferred.
2. **Fetch and parse the root.** `get(root_url)`, then parse into a
   `SitemapDocument`. A Shopify root lists child sitemaps; a flat sitemap lists
   `urls` directly.
3. **Fetch and parse each child.** For every child sitemap, `get(location)` and
   parse its `SitemapUrl` entries (with images, change frequency, and priority
   when present) into a child `SitemapDocument`. `kind()` labels each by category.
4. **Assemble.** Attach the parsed children under the root's `children` and return
   the root.

**Fallback:** if the root has no child sitemaps and lists `urls` directly,
return it as a single-node tree. This keeps non-Shopify or flat sitemaps
working.

---

## 4. Dependency contract

`sitemap_discovery` does not fetch HTTP itself — it calls the client that
`retailer_data_ingestion` provides. That client does not exist yet; this is the
interface we expect it to expose:

```rust
client.get(url: &str) -> Result<String, FetchError>   // response body as text
```

`fetch_sitemap` calls `get` once for the root and once per child sitemap.

**Open coupling question:** if the client ends up `async`, `fetch_sitemap`
becomes `async` to match. The shape above assumes a synchronous client for now.

---

## 5. Errors

```rust
pub enum SitemapError {
    Fetch(/* underlying client error */),  // get() failed
    Parse(/* what/where */),               // malformed or unexpected XML
    UnknownRetailer,                       // no root sitemap URL known for this retailer
}
```

`UnknownRetailer` is unreachable while `Minisforum` is the only code, but keeps
the API honest as retailers are added. Parsing a `<changefreq>` value surfaces
`ParseChangeFrequencyError` (defined in `sitemap.rs`), which maps under `Parse`.

---

## 6. Scope

**In scope**

- Fetch the root sitemap and all child sitemaps.
- Parse them into one composite `SitemapDocument` tree (URL entries, images,
  change frequency, priority, lastmod, nested children).
- Classify each document by URL (`SitemapKind`) and expose recursive queries
  (`all_urls`, `urls_of_kind`).
- Return the tree.

**Out of scope**

- Persisting to PostgreSQL (the caller stores the result — Stage B).
- Classifying individual page URLs as product / collection / catalog (Stage C) —
  distinct from the child-sitemap-file classification `SitemapKind` does.
- `robots.txt` discovery, retries, concurrency, and anti-scraping (deferred).

---

## 7. Dependencies & assumptions

- **New dependency:** `quick-xml` for parsing sitemap XML.
- Assumes `get()` returns decoded UTF-8 text.
- Assumes the retailer's root sitemap URL is known and hardcoded in Phase 0.

---

## 8. Next step

The composite data model is implemented in
`apps/mvp/src/sitemap_discovery/sitemap.rs` (with unit tests for `SitemapKind`
classification, `ChangeFrequency` parsing, and the recursive tree queries).
Still to build: the `RetailerCode` → root-URL resolver, the XML parser that
produces `SitemapDocument`, and `fetch_sitemap` wiring it to
`retailer_data_ingestion`'s client — then tests against a sample Shopify
sitemap.
