.PHONY: help fix cfix clippy fmt check build buildr qbuild test qtest lint clean wip align-markdown-table-columns

# Workspace folders that contain Makefiles
WORKSPACES := shared

help:
	@echo "Available targets:"
	@echo "  fix                        Auto-fix issues with cargo fix"
	@echo "  cfix                       Clippy fix with --allow-dirty"
	@echo "  clippy                     Lint check"
	@echo "  fmt                        Format code"
	@echo "  check                      Check compilation"
	@echo "  build                      Build all workspaces"
	@echo "  buildr                     Release build"
	@echo "  qbuild                     Quick build (suppress warnings)"
	@echo "  test                       Run all tests"
	@echo "  qtest                      Quick test (suppress warnings)"
	@echo "  lint                       Alias for clippy"
	@echo "  clean                      Remove build artifacts"
	@echo "  wip                        Work-in-progress commit"
	@echo "  align-markdown-table-columns  Align markdown table columns"

fix:
	set -e; for ws in $(WORKSPACES); do \
		echo "==> $$ws"; \
		$(MAKE) -C $$ws fix; \
	done

cfix:
	set -e; for ws in $(WORKSPACES); do \
		echo "==> $$ws"; \
		$(MAKE) -C $$ws cfix; \
	done

clippy:
	set -e; for ws in $(WORKSPACES); do \
		echo "==> $$ws"; \
		$(MAKE) -C $$ws clippy; \
	done

fmt:
	set -e; for ws in $(WORKSPACES); do \
		echo "==> $$ws"; \
		$(MAKE) -C $$ws fmt; \
	done

check:
	set -e; for ws in $(WORKSPACES); do \
		echo "==> $$ws"; \
		$(MAKE) -C $$ws check; \
	done

build:
	set -e; for ws in $(WORKSPACES); do \
		echo "==> $$ws"; \
		$(MAKE) -C $$ws build; \
	done

buildr:
	set -e; for ws in $(WORKSPACES); do \
		echo "==> $$ws"; \
		$(MAKE) -C $$ws buildr; \
	done

qbuild:
	set -e; for ws in $(WORKSPACES); do \
		echo "==> $$ws"; \
		$(MAKE) -C $$ws qbuild; \
	done

test:
	set -e; for ws in $(WORKSPACES); do \
		echo "==> $$ws"; \
		$(MAKE) -C $$ws test; \
	done

qtest:
	set -e; for ws in $(WORKSPACES); do \
		echo "==> $$ws"; \
		$(MAKE) -C $$ws qtest; \
	done

lint: clippy

clean:
	set -e; for ws in $(WORKSPACES); do \
		echo "==> $$ws"; \
		$(MAKE) -C $$ws clean; \
	done

wip:
	git add TODO.md && git commit -m '** TODO **' || printf "No todo to commit"
	git add . && git commit -am 'wip'

align-markdown-table-columns:
	@cd dev/align-markdown-table-columns && go build -o align-markdown-table-columns .
	@./dev/align-markdown-table-columns/align-markdown-table-columns ./docs
