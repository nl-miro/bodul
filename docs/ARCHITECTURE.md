# Bodul Architecture Overview

## Services

### Retailer Management

Source of truth for retailer information: name, locations, settings, status, and other metadata.

### Retailer Sourcing

Uses projected data from Retailer Management to determine how to retrieve information from retailer web or API. Handles all retrieval and parsing of product and pricing information from retailers.

### Retailer Data Ingestion

Receives parsed data from Retailer Sourcing and prepares it for storage. Responsibilities include:
- Data validation to ensure quality standards
- Deduplication of products and offers
- Conflict resolution for duplicate products with different identifiers
- Batching and scheduling of ingestion runs
- Error handling, retry logic, and audit logging

### Product Information Management

Source of truth for product data. Holds comprehensive product information including names, descriptions, attributes, categories, and cross-retailer product relationships.

### Retailer Offer

Holds offer and availability information for products at specific retailers. Manages pricing, current availability status, and retailer-specific product details.


## System Components

### Data Ingestion Layer

> TODO: Define scope and responsibilities for data ingestion layer

### Data Processing & Normalization

### Storage Layer

### API Layer

### Frontend

## Data Models

### Product Model

### Offer/Price Model

### Price History Model

## Integration Points

### Retailer Integrations

### External Services

## Deployment & Infrastructure

## Technology Stack

## Design Decisions

## Future Considerations
