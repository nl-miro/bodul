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
  - catalog pages
  - catalog page items
  - product pages
- storing images 


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
