# Tasks



## DONE

- add poem to retailer-sourcing app
- create health check endpoint
- create lib retailer parsing so we can iterate faster without compiling diesel + serde
- create lib retailer guild so we can keep RetailerCode enum there
- rewrite integration tests so we can quickly switch between cleaning up by them selves or not
- fix tests that could not run due to side effects of other tests
- creating binaries to truncate the database (one for main, other one for testing)


## IN PROGRESS


## TODO


## BACKLOG

- deploy margo to internet
- support error information on inbox,outbox, commanding and eventing entries
- build tracing by correlation id and causation id for specific or for all types

- keep a fresh command id for every daily sourcing request
- use mulac kernel to wire up the daily sourcing endpoint and command pipeline
- add a concrete database-backed integration test harness for daily sourcing verification



## DONT FORGET
