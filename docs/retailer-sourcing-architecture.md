# Retailer Sourcing Architecture

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
