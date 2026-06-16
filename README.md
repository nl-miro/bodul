# bodul

## Phase 1

Phase 1 focuses on building a reliable product discovery pipeline for multiple
Minisforum storefronts, all of them running on Shopify.

```text
                        +------------------------------+
| Trigger updates |
                        +------------------------------+
|-----|
                                       v
                      +----------------+----------------+
|-----|
                      v                                 v
       +------------------------------+  +------------------------------+
| Discover sitemap files |     | Discover catalog leaves |
| and URLs               |     | and URLs                |
       +------------------------------+  +------------------------------+
|-----|
                      +----------------+----------------+
|-----|
                                       v
                        +------------------------------+
| Fetch product pages |
                        +------------------------------+
|-----|
                                       v
                        +------------------------------+
| Process retrieved data |
                        +------------------------------+
|-----|
                                       v
                        +------------------------------+
| Classify and match products |
                        +------------------------------+
|-----|
                      +----------------+----------------+
|-----|
                      v                                 v
       +------------------------------+  +------------------------------+
| Product catalog matched or |     | Store catalog persisted |
| created                    |     |                         |
       +------------------------------+  +------------------------------+
```

Discovery uses two paths:

- Sitemap-based discovery.
  - Fetch `sitemap.xml`.
  - Extract all sitemap files.
  - Ignore non-default locales.
  - Download sitemap files and store them in the database.
  - Iterate over catalog pages to collect quick price checks, availability, and newly discovered products.
  - Iterate over product pages to scrape images, descriptions, availability, and other core product details.

```text
                        +------------------------------+
| Fetch sitemap.xml |
                        +------------------------------+
|-----|
                                       v
                        +------------------------------+
| Extract sitemap files |
                        +------------------------------+
|-----|
                                       v
                        +------------------------------+
| Ignore non-default locales |
                        +------------------------------+
|-----|
                                       v
                        +------------------------------+
| Download and store sitemaps |
                        +------------------------------+
|-----|
                                       v
                      +----------------+----------------+
|-----|
                      v                                 v
       +------------------------------+  +------------------------------+
| Iterate catalog pages |     | Iterate product pages |
       +------------------------------+  +------------------------------+
|-----|
                      v                                 v
       +------------------------------+  +------------------------------+
| Price and availability    |     | Images, descriptions, and |
| Newly discovered products |     | availability              |
       +------------------------------+  +------------------------------+
```

- Web scraping fallback when sitemap coverage is incomplete.
  - Iterate over the site menu to gather catalog leaves.
  - Iterate over catalog pages to collect lightweight product details and product URLs.
  - Visit product pages in order and scrape images, descriptions, availability, and other core product details.

```text
                        +------------------------------+
| Iterate site menu |
                        +------------------------------+
|-----|
                                       v
                        +------------------------------+
| Gather catalog leaves |
                        +------------------------------+
|-----|
                                       v
                        +------------------------------+
| Iterate catalog pages |
                        +------------------------------+
|-----|
                                       v
                        +------------------------------+
| Collect product details |
| and URLs                |
                        +------------------------------+
|-----|
                                       v
                        +------------------------------+
| Visit product pages |
| in order            |
                        +------------------------------+
|-----|
                                       v
                        +------------------------------+
| Scrape images and          |
| descriptions, availability |
                        +------------------------------+
```

## Processing retrieved catalog data

Each fetched catalog page is sliced into individual product items and each slice
is processed independently.

```text
                        +------------------------------+
| Retrieved catalog page |
                        +------------------------------+
|-----|
                                       v
                        +------------------------------+
| Slice into product items |
                        +------------------------------+
|-----|
                              for each item
|-----|
                                       v
                        +------------------------------+
| Extract mandatory fields   |
| product_url, title, price, |
| image                      |
                        +------------------------------+
|-----|
                                       v
                        +------------------------------+
| Extract optional fields |
| id, category, and more  |
                        +------------------------------+
|-----|
                                       v
                        +------------------------------+
| Create product record |
                        +------------------------------+
|-----|
                                       v
                        +------------------------------+
| Emit product event |
                        +------------------------------+
```

Mandatory fields:

- `product_url`
- `title`
- `price`
- `image`

Optional fields (list to grow during implementation as gaps are found):

- `id`
- `category`

Updates support multiple strategies.

- Use the daily update run to decide which strategies to apply.
- Define separate checks for regular price updates and availability updates.
- Store locale-based descriptions for each product.
