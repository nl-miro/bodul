# Retailer Sourcing — Flow Overview

How the retailer-sourcing service decides what to source and turns that into
per-retailer work. This is a living document; it grows one step at a time as
the flow is built out.

## Concepts

- **Retailer** — a shop we source product data from, identified by a
  `RetailerCode` (e.g. `MinisForumEU`).
- **Sourcing method** — how we source a given retailer. Two methods exist:
  - `Sitemap` — discover URLs via the retailer's sitemap.
  - `WebScraping` — discover/parse via the retailer's web pages.
- A retailer is sourced **per method**: one retailer can be sourced by both
  sitemap and web scraping independently.
- **FetchAndExtract** — the shared building block the pipelines below compose:
  given a URL and an *extractor*, it fetches the page and runs the extractor on
  the response to produce structured data. The pipelines differ only in which
  extractor they supply:
  - product details page → extractor `ExtractProductDetailsData`
  - category/listing page → extractor `SliceProducts → ExtractCategoryProductData`

## Step 1 — Start daily sourcing

The entry point. Decides *which* sourcing work should happen today and fans it
out into one job per (retailer, method).

```
StartDailySourcing (command)
        │
        ▼
DailySourcingHandler
        │  finds all active retailers
        ▼
DailySourcingRequested (event)   ── carries every active RetailerCode
        │
        ▼
DailySourcingFanOutSubscriber
        │  for each retailer × each method
        ▼
SourceRetailer (command)         ── one per (retailer_code, method)
```

1. **`StartDailySourcing`** is the trigger command (today: dispatched by the
   `POST /daily-sourcing/` endpoint).
2. **`DailySourcingHandler`** handles it: it looks up the set of currently
   active retailers and emits a single **`DailySourcingRequested`** event
   carrying all of their retailer codes.
3. **`DailySourcingFanOutSubscriber`** subscribes to
   `DailySourcingRequested` and creates one **`SourceRetailer`** command for
   every `(retailer_code, method)` combination — i.e. it expands the announced
   retailers across all sourcing methods.

A `SourceRetailer` command targets exactly one retailer with exactly one
method. With N active retailers and M methods, one `StartDailySourcing`
produces 1 `DailySourcingRequested` event and N×M `SourceRetailer` commands.

After this step the `SourceRetailer` commands are persisted and waiting. Each
one is handled by the discovery feature matching its method (Step 2).

## Step 2 — Discovery features

A `SourceRetailer` command is picked up by the discovery feature for its
method. There is one feature per `SourcingMethod`, each a self-contained
top-level feature (its own module in `lib.rs` with a public `io` surface):

- **SitemapDiscovery** — handles `SourceRetailer { method: Sitemap }`.
- **WebsiteDiscovery** — handles `SourceRetailer { method: WebScraping }`.

Each feature owns one slice of the flow: take its `SourceRetailer` command,
fetch the raw source from the retailer, persist it, and announce that the
source was retrieved.

### SitemapDiscovery

Starts by fetching the retailer's sitemap files. Each fetched sitemap is then
sent to two independent enumerations: **ProductEnumeration** and
**CategoryEnumeration**.

```
SourceRetailer { method: Sitemap }
        │
        ▼
fetch sitemap files
        │
        ├──────────────► ProductEnumeration
        │                     │  for every product URL in the sitemap
        │                     ▼
        │                 FetchAndExtract
        │                   (extractor: ExtractProductDetailsData)
        │
        └──────────────► CategoryEnumeration
                              │  for every category found in the sitemap,
                              │  page first → last
                              ▼
                          FetchAndExtract
                            (extractor: SliceProducts
                                        → ExtractCategoryProductData)
```

**ProductEnumeration** — enumerates the individual product URLs in the
sitemap. For each product it runs **FetchAndExtract** with the
`ExtractProductDetailsData` extractor, pulling that product's data out of its
fetched details page.

**CategoryEnumeration** — enumerates the category URLs in the sitemap. For
each category it pages through the listing from the first page to the last,
running **FetchAndExtract** on each page with the
`SliceProducts → ExtractCategoryProductData` extractor — **SliceProducts**
splits the page into its individual products and **ExtractCategoryProductData**
pulls the product data available on the listing.

The two enumerations are complementary sources of product data: product
details pages give the full per-product view, while category listings give
the per-category view of the products they contain.

### WebsiteDiscovery

Starts by fetching the retailer's homepage, then builds the category structure
from the site's menu and walks it.

```
SourceRetailer { method: WebScraping }
        │
        ▼
fetch homepage
        │
        ▼
ProcessMenu  (build a category tree from the menu)
        │
        ▼
for every node in the category tree:
FetchNode
        │  page first → last
        ▼
FetchAndExtract
  (extractor: SliceProducts → ExtractCategoryProductData)
```

**ProcessMenu** — parses the homepage's navigation menu into a **category
tree**.

**FetchNode** — for every node in the tree it pages through the node's listing,
running **FetchAndExtract** on each page with the
`SliceProducts → ExtractCategoryProductData` extractor — **SliceProducts**
splits the page into its individual products and **ExtractCategoryProductData**
pulls the products on that listing.

Note the fetch+extract step is shared with SitemapDiscovery's
CategoryEnumeration: both run **FetchAndExtract** with the same
`SliceProducts → ExtractCategoryProductData` extractor. Only *category
discovery* differs — WebsiteDiscovery derives categories from the homepage menu
(a tree), while CategoryEnumeration derives them from the sitemap (a flat
list).

## Later steps

_To be documented._ What the extraction steps produce — the shape of the data
that **ExtractProductDetailsData** and **ExtractCategoryProductData** emit, and
where it goes next (persisted product records, downstream events, etc.) — will
be described here as it is designed.
