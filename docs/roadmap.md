# Product Roadmap

| Field        | Value      |
|--------------|------------|
| Last updated | 2026-01-14 |
| Owner(s)     | @miro      |
| Reviewer     | @miro      |

## Launch Goals

| #   | Goal                                         | Status  | Notes                  |
|-----|----------------------------------------------|---------|------------------------|
| 1   | Collect catalog page item data for retailers | Planned | Core data ingestion    |
| 2   | Collect product page data from retailers     | TBD     | Detailed product info  |
| 3   | Data normalization                           | TBD     | Unified data model     |
| 4   | Ingest data into inventory                   | TBD     | Populate inventory DB  |
| 5   | Create products from inventory               | TBD     | Product catalog        |
| 6   | Implement mapping of products to inventory   | TBD     | Product-inventory link |
| 7   | Product classification                       | TBD     | Categorization/tagging |
| 8   | Price tracking                               | TBD     | Historical pricing     |
| 9   | Availability tracking                        | TBD     | Stock status           |
| 10  | Minimal customer facing website & mobile app | TBD     | MVP frontend           |
| 11  | Auth service                                 | TBD     | Internal service auth  |

## Phase 2 Goals

| #   | Goal | Status | Notes |
|-----|------|--------|-------|
| 1   |      |        |       |


## Sub-goals Breakdown

### Launch Goal 1: Collect catalog page item data for retailers

Build the data foundation for a Croatian & Slovenian price comparison platform—starting with electronics retailers—so consumers can find products, compare prices, and spot fake "sales" through price history tracking.

- [x] Define target retailer list (initial set) [PR-0001: Target Retailers](product/requirements/PR-0001-target-retailers.md)
- [x] Internal service security [EDR-0006: Internal Service Security](technical/decisions/EDR-0006-internal-service-security.md)
- [ ] Build page crawler/scraper
- [ ] Handle pagination and category navigation
- [ ] Store raw data in PostgreSQL
- [ ] Handle rate limiting and anti-bot measures
- [ ] Schedule recurring data collection

### Launch Goal 2: Collect product page data from retailers


### Launch Goal 3: Data normalization


### Launch Goal 4: Ingest data into inventory


### Launch Goal 5: Create products from inventory


### Launch Goal 6: Implement mapping of products to inventory


### Launch Goal 7: Product classification


### Launch Goal 8: Price tracking


### Launch Goal 9: Availability tracking


### Launch Goal 10: Minimal customer facing website & mobile app


### Launch Goal 11: Auth service

Centralized HTTP auth service for internal service-to-service authentication. This is the upgrade path from the current `auth_client` library (EDR-0006 Option 3) to a centralized solution (Options 1 or 2) when the platform scales and requires dynamic credential management.

Note: The current `auth_client` library with hardcoded credentials is sufficient for early-phase development and is already implemented as part of Launch Goal 1.
