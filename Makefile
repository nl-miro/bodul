.PHONY: align-markdown-table-columns

align-markdown-table-columns:
	@cd dev/align-markdown-table-columns && go build -o align-markdown-table-columns .
	@./dev/align-markdown-table-columns/align-markdown-table-columns ./docs
