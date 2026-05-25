.PHONY: align-markdown-table-columns wip fmt check test

align-markdown-table-columns:
	./etc/dev/align-markdown-table-columns/align-markdown-table-columns .

wip:
	git add . && git commit -am 'wip'

fmt:
	cd apps/retailer-management && make fmt
	cd apps/retailer-sourcing && make fmt
	cd apps/retailer-data-ingestion && make fmt
	cd apps/product-information-management && make fmt
	cd apps/retailer-offer && make fmt
	cd lib/retailer-guild && make fmt
	cd lib/retailer-parsing && make fmt

check:
	cd apps/retailer-management && make check
	cd apps/retailer-sourcing && make check
	cd apps/retailer-data-ingestion && make check
	cd apps/product-information-management && make check
	cd apps/retailer-offer && make check
	cd lib/retailer-guild && make check
	cd lib/retailer-parsing && make check

test:
	cd apps/retailer-management && make test
	cd apps/retailer-sourcing && make test
	cd apps/retailer-data-ingestion && make test
	cd apps/product-information-management && make test
	cd apps/retailer-offer && make test
	cd lib/retailer-guild && make test
	cd lib/retailer-parsing && make test
