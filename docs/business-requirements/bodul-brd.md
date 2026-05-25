# Bodul Business Requirements Document

## Overview
Bodul is a platform for finding the best deals, tracking prices over time, and investigating products across a limited set of retailers in the initial phase.

This first phase is intentionally narrow: validate the core product flow with a small retailer set before expanding coverage.

## Business Goal
Help users quickly compare product prices, monitor price changes, and research products across supported retailers.

## Phase 1 Scope
The initial release will focus on:
- Ingesting product and pricing data from a small number of retailers
- Normalizing product data into a shared model
- Tracking current and historical prices
- Supporting basic product investigation views
- Creating a foundation that can be extended to more retailers later

## Supported Retailers
Initial retailer list:

| Retailer ID    | Retailer Name | URL                             |
|----------------|---------------|---------------------------------|
| `MinisForumEu` | MinisForumEU  | `https://minisforumpc.eu/`      |
| `MinisForumUs` | MinisForumUS  | `https://store.minisforum.com/` |
| `MinisForumUk` | MinisForumUK  | `https://www.minisforum.uk/`    |
| `MinisForumFr` | MinisForumFR  | `https://minisforumpc.fr/`      |
| `MinisForumCa` | MinisForumCA  | `https://ca.minisforum.com/`    |
| `MinisForumAu` | MinisForumAU  | `https://au.minisforum.com/`    |

## Core Requirements

### Retailer ingestion
- The system must be able to ingest catalog data from the listed retailers.
- The system must capture at minimum:
  - product name
  - current price
  - old price or discount, if available
  - availability
  - product URL
  - image URL
  - SKU or another stable identifier when available

### Price tracking
- The system must store price history for each tracked product.
- The system must support identifying price drops and promotions over time.

### Product investigation
- The system must allow a user to inspect a product and see its current offer across supported retailers.
- The system should surface price history and retailer-specific details where available.

### Data normalization
- Retailer-specific feeds must map into a shared product and offer model.
- The system should tolerate missing fields when a retailer does not expose all desired attributes.

## Non-Goals For Phase 1
- Full marketplace coverage
- Advanced recommendation ranking
- User accounts and personalized alerts
- Manual seller onboarding tooling
- International expansion beyond the listed MinisForum storefronts

## Success Criteria
- The first retailer integrations can be ingested reliably.
- Product pages show current pricing and basic history.
- The data model can support adding new retailers without redesign.

## Assumptions
- Retailer storefronts may differ in structure and available metadata.
- The first version prioritizes a working end-to-end pipeline over perfect completeness.
- Retailer integrations will be added incrementally.

