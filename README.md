# bodul

## Phase 1

Phase 1 focuses on building a reliable product discovery pipeline for multiple Minisforum storefronts, all of them running on Shopify.

- Source product data from several Minisforum sites.
- Build discovery through two paths:
  - Sitemap-based discovery.
    - Fetch `sitemap.xml`.
    - Extract all sitemap files.
    - Ignore non-default locales.
    - Download sitemap files and store them in the database.
    - Iterate over catalog pages to collect quick price checks, availability, and newly discovered products.
    - Iterate over product pages to scrape images, descriptions, availability, and other core product details.
  - Web scraping fallback when sitemap coverage is incomplete.
    - Iterate over the site menu to gather catalog leaves.
    - Iterate over catalog pages to collect lightweight product details and product URLs.
    - Visit product pages in order and scrape images, descriptions, availability, and other core product details.
