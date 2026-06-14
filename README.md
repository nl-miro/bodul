# bodul

## Phase 1

Phase 1 focuses on building a reliable product discovery pipeline for multiple Minisforum storefronts, all of them running on Shopify.

```text
                             +-------------------+
                             |  Trigger updates  |
                             +---------+---------+
                                       |
                                       v
                 +---------------------+---------------------+
                 |                                           |
                 v                                           v
        +-------------------+                       +-------------------+
        | Discover sitemap  |                       | Discover catalog  |
        |  files and URLs   |                       |  leaves and URLs  |
        +---------+---------+                       +---------+---------+
                  |                                           |
                  +---------------------+---------------------+
                                        |
                                        v
                             +---------------------+
                             |    Fetch product    |
                             |    detail pages     |
                             +----------+----------+
                                        |
                                        v
                             +---------------------+
                             |  Process retrieved  |
                             |     information     |
                             +----------+----------+
                                        |
                                        v
                             +---------------------+
                             | Classify and match  |
                             |      products       |
                             +----------+----------+
                                        |
                                        v
                 +----------------------+----------------------+
                 |                                             |
                 v                                             v
        +--------------------+                       +--------------------+
        |  Product catalog   |                       |   Store catalog    |
        | matched or created |                       |     persisted      |
        +--------------------+                       +--------------------+
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
                   |      Fetch sitemap.xml       |
                   +--------------+---------------+
                                  |
                                  v
                   +------------------------------+
                   |  Extract all sitemap files   |
                   +--------------+---------------+
                                  |
                                  v
                   +------------------------------+
                   |  Ignore non-default locales  |
                   +--------------+---------------+
                                  |
                                  v
                   +------------------------------+
                   | Download and store sitemaps  |
                   +--------------+---------------+
                                  |
                                  v
               +------------------+-------------------+
               |                                      |
               v                                      v
+------------------------------+       +------------------------------+
|    Iterate catalog pages     |       |    Iterate product pages     |
+--------------+---------------+       +--------------+---------------+
               |                                      |
               v                                      v
+------------------------------+       +------------------------------+
|    Price and availability    |       |  Images, descriptions, and   |
|  Newly discovered products   |       |         availability         |
+------------------------------+       +------------------------------+
```

- Web scraping fallback when sitemap coverage is incomplete.
  - Iterate over the site menu to gather catalog leaves.
  - Iterate over catalog pages to collect lightweight product details and product URLs.
  - Visit product pages in order and scrape images, descriptions, availability, and other core product details.

```text
+------------------------------+
|      Iterate site menu       |
+--------------+---------------+
               |
               v
+------------------------------+
|    Gather catalog leaves     |
+--------------+---------------+
               |
               v
+------------------------------+
|    Iterate catalog pages     |
+--------------+---------------+
               |
               v
+------------------------------+
|   Collect product details    |
|           and URLs           |
+--------------+---------------+
               |
               v
+------------------------------+
|     Visit product pages      |
|           in order           |
+--------------+---------------+
               |
               v
+------------------------------+
|        Scrape images,        |
|  descriptions, availability  |
+------------------------------+
```

Updates support multiple strategies.

- Use the daily update run to decide which strategies to apply.
- Define separate checks for regular price updates and availability updates.
- Store locale-based descriptions for each product.
