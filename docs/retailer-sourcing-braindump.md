# Retailer Sourcing Architecture


## Workflows

### Menu based

#### Step 1: Catalog tree sourcing
- Fetch source for building catalog tree
- Build catalog tree from source
- Store catalog tree in database

#### Step 2: Catalog page sourcing
- Process all leaves of catalog tree to build catalog pages
- Fetch catalog page 1
- Extract following catalog page numbers, send them for fetching
- Store catalog pages in database
- Slice catalog page into catalog page items
- Store catalog page items in database
- Store product page urls in database

#### Step 3: Product page sourcing
- Fetch product page
- Store product page in database


## Challanges

- storing vast amounts of data
    - catalog pages - 3GB/day for first 30 retailers
    - catalog page items - 2GB/day for first 30 retailers
    - product pages
- storing images


## Possible Solutions

### All in one

Vendor sourcing app would
- dirigate all processes
- fetch external data sources
- store data in database
- process data

Questions:
- how to achieve retailer targeting?
- storing images?

Pros:

- Everything is in one place
- Data is stored in one place
- Data archive and backups can be done manually
- Easier reporting

Cons:

- Possibility to create unmantaitable amount of code and data
- makes scaling harder

### Separate apps per retailer

While code would be mostly shared, each app would have it's own binary and database.

Questions:
- storing images?
-
Pros:
- Must automate archive and backups

Cons:
- Fetching reports would mean sending 30 requests at once to 30 different apps




### Separate apps per function

Vendor sourcing app would
- dirigate all processes
- process data

Fetcher app would
- fetch external data sources

Central data storage app would
- store data in database

Questions:
- how to achieve retailer targeting?
- storing images?

Pros:
- Must automate archive and backups

Cons:
- Fetching reports would mean sending 30 requests at once to 30 different apps


### Event-driven with raw data storage

Queue-based job distribution with raw data lake.

Vendor sourcing app would
- orchestrate workflows
- push fetch jobs to queue
- process extracted data

Queue (Redis Streams / RabbitMQ) would
- distribute jobs to workers
- handle retries and dead letters
- rate limit per retailer

Fetch workers would
- consume jobs from queue
- fetch external data sources
- store raw HTML/JSON to object storage (S3/MinIO)
- push parse jobs to queue

Parse workers would
- consume parse jobs from queue
- extract structured data from raw storage
- store structured data in database

Questions:
- object storage setup (S3/MinIO)?
- queue infrastructure choice?

Pros:
- Reprocessing is easy (schema changes, bug fixes)
- Natural scaling - add/remove workers as needed
- Built-in retry and backpressure handling
- Raw data serves as audit trail
- Retailer-specific rate limiting at queue level

Cons:
- Additional infrastructure (message broker, object storage)
- Eventual consistency
- Debugging distributed jobs is harder







```
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//




















```






## Features

- Retailer catalog tree sourcing
    - Fetch source for building catalog tree
    - Build catalog tree from source
    - Store catalog tree in database and on disk?
- Retailer catalog page sourcing
    - Process all leaves of catalog tree to build catalog pages
    - Fetch catalog page 1
    - Extract following catalog page numbers, send them for fetching
    - Slice catalog page into catalog page items


## Workflows


1. Scheduler: Request catalog tree refresh for specific or all retailers
    - Request lands into inbox
2. System: Read inbox, process request for catalog tree refresh, store all fetch url requests to outbox
3. Saga: Read outbox, take each fetch url request and push it



## Project Structure

```
retailer_sourcing/
├── domain/
│   ├── retriever/
│   ├── crawler/
│   └── scraper/
├── docs/
├── dev/
├── Makefile
└── README.md
```
