# Business Requirements Skeleton

Use this skeleton to shape the Bodul business requirements document. The goal is
to answer the questions before writing the final BR, then collapse the answers
into a clean stakeholder-facing document.

## 1. Executive Summary

- What problem is Bodul solving?
- Who has this problem today?
- What decisions should Bodul make easier?
- What happens if Bodul is not built?
- What does success look like in 1 month, 3 months, and 6 months?

## 2. Users and Stakeholders

- Who are the primary users?
- Are users technical, operational, commercial, executive, or client-facing?
- What are their top workflows?
- What information do they need first?
- What actions should they be able to take from the system?
- Who owns the business outcome?
- Who owns operational follow-up when data quality or scraping fails?

## 3. Business Goals

- What business outcomes should Bodul improve?
- Which decisions should become faster, cheaper, or more reliable?
- Which metrics prove the project is working?
- Which manual workflows should be reduced or removed?
- Which risks should the project reduce?

## 4. Scope

- Which storefronts are in scope?
- Are only Minisforum Shopify storefronts in scope, or should more brands be
  supported later?
- Which product types are in scope?
- Are accessories, bundles, variants, refurbished products, and out-of-stock
  products included?
- What is explicitly out of scope for the first release?
- What should be deferred to a later phase?

## 5. Data Requirements

- What product data must be collected?
- What data is nice to have?
- What fields define a unique product?
- How should variants be represented?
- How should locale-specific descriptions be stored?
- How should missing, conflicting, or stale data be handled?
- What is the source of truth when storefronts disagree?

## 6. Discovery Requirements

- How often should product discovery run?
- Should sitemap discovery be the primary path?
- When should menu scraping fallback run?
- What counts as incomplete sitemap coverage?
- Should deleted or discontinued products be detected?
- Should new products trigger alerts, reports, or downstream events?

## 7. Update Requirements

- How often should price updates run?
- How often should availability updates run?
- Should full detail scraping run daily, weekly, or only when changes are
  detected?
- What update latency is acceptable?
- Which fields require history tracking?
- Should the system detect price drops, restocks, discontinued products, or
  renamed products?

## 8. Matching and Classification

- What makes two storefront listings the same product?
- Which attributes should be used for matching: SKU, title, handle, specs, or
  images?
- How confident must matching be before automatic linking?
- What happens when matching is uncertain?
- Which product categories or classes are required?
- Who reviews bad or uncertain matches?

## 9. Outputs and Consumers

- What should Bodul produce: database records, API responses, dashboard views,
  CSV exports, reports, or alerts?
- Who consumes each output?
- What format do downstream systems need?
- Are exports scheduled or on demand?
- What are the most important views or reports?

## 10. Quality Requirements

- What accuracy level is required for prices?
- What accuracy level is required for availability?
- How fresh must data be?
- How much scraping failure is acceptable?
- How should failures be reported?
- What audit trail is required?

## 11. Compliance and Risk

- Are there scraping, legal, or commercial constraints?
- Should robots.txt and storefront rate limits be respected?
- Are there data retention requirements?
- Are credentials, proxies, or API keys involved?
- What should happen if a storefront blocks scraping?

## 12. Operations

- Who owns monitoring?
- What alerts are needed?
- What are the critical failure modes?
- How should retries work?
- What manual recovery actions are expected?
- What logs or debug artifacts must be stored?

## 13. Milestones

- What is the MVP?
- What must be true before Phase 1 is considered complete?
- What comes after Phase 1?
- What can be deferred?
- What are the main dependencies or blockers?

## Suggested Final BR Structure

1. Executive Summary
2. Business Goals
3. Users and Stakeholders
4. Scope
5. Functional Requirements
6. Data Requirements
7. Non-Functional Requirements
8. Success Metrics
9. Risks and Assumptions
10. Milestones
