# TS-0001: Makefile Structure and Targets

| Field         | Value                                                                |
|---------------|----------------------------------------------------------------------|
| Status        | Active                                                               |
| Owner         | @miro                                                                |
| Created       | 2026-01-14                                                           |
| Last updated  | 2026-01-14                                                           |
| Related issue | [BOD-4: Add Makefile tasks](https://linear.app/nulllabs/issue/BOD-4) |

## Summary

Specification for Bodul Makefile structure, iteration model, and target definitions so every workspace can share the same commands.

## Intro: new teammate quickstart

- Tools: `make` plus Rust toolchain with `cargo`.
- Where to run:
  - Repo root: `make <target>` iterates all workspaces in `WORKSPACES` (currently `shared`).
  - Workspace/crate: `cd shared` or `cd shared/elemental` then `make <target>` to run only there.
- First day path:
  1) `make help` (root and workspace) to see available targets.
  2) `make fmt cfix test` for a clean baseline.
  3) `make build` or `make buildr` before pushing.
- Warning control: `qbuild`/`qtest` suppress warnings for fast loops; standard targets enforce warnings.
- WIP safety: `make wip` is root-only and performs git commits—skip in CI.

## Makefile layout

### Root Makefile (`Makefile`)
- Purpose: orchestrate build/test/dev tasks across workspaces.
- Workspace list:
```makefile
WORKSPACES := shared
```
- Iteration pattern used by all root targets:
```makefile
target:
	set -e; for ws in $(WORKSPACES); do \
		echo "==> $$ws"; \
		$(MAKE) -C $$ws target; \
	done
```
- Behavior: fail fast on the first workspace error (`set -e`); echo workspace names for traceability.

### Workspace / crate Makefiles (e.g., `shared/Makefile`, `shared/elemental/Makefile`)
- Purpose: implement the commands (usually `cargo ...`) for that scope.
- Scope rule: workspace-level uses `--workspace`; crate-level uses `-p <crate>`.
- Must include a `help` target listing the local targets and descriptions.

## Target catalog (authoritative spec)

Declare all targets as `.PHONY` and implement them in every workspace Makefile. Root targets iterate `$(WORKSPACES)` unless marked direct.

| Target   | Category     | Root behavior             | Workspace / crate command                                                          | Notes                                                          |
|----------|--------------|---------------------------|------------------------------------------------------------------------------------|----------------------------------------------------------------|
| `fmt`    | Code quality | Iterate workspaces        | `cargo fmt`                                                                        | Run before commits.                                            |
| `clippy` | Code quality | Iterate workspaces        | `cargo clippy --all-targets`                                                       | Use `-D warnings` to fail on warnings.                         |
| `cfix`   | Code quality | Iterate workspaces        | `cargo clippy --fix --allow-dirty`                                                 | Auto-fix lint; review diffs.                                   |
| `fix`    | Code quality | Iterate workspaces        | `cargo fix --lib --allow-dirty`                                                    | Compiler-suggested fixes.                                      |
| `lint`   | Code quality | Iterate workspaces        | Alias calling `clippy` target                                                      | Keep for familiarity.                                          |
| `check`  | Build        | Iterate workspaces        | `cargo check`                                                                      | Fast validation without artifacts.                             |
| `build`  | Build        | Iterate workspaces        | `cargo build`                                                                      | Debug build.                                                   |
| `buildr` | Build        | Iterate workspaces        | `cargo build --release`                                                            | Release build.                                                 |
| `qbuild` | Build        | Iterate workspaces        | `RUSTFLAGS="-Awarnings" cargo build`                                               | Suppress warnings for quick loops.                             |
| `test`   | Testing      | Iterate workspaces        | `cargo test`                                                                       | Full suite.                                                    |
| `qtest`  | Testing      | Iterate workspaces        | `RUSTFLAGS="-Awarnings" cargo test`                                                | Suppress warnings for speed.                                   |
| `clean`  | Maintenance  | Iterate workspaces        | `cargo clean`                                                                      | Clears `target/`.                                              |
| `wip`    | Maintenance  | **Direct** (no iteration) | Git ops: `git add TODO.md && git commit ...` + `git add . && git commit -am 'wip'` | In all Makefiles; root version direct to avoid nested commits. |
| `help`   | Discovery    | Iterate workspaces        | Print local targets and short descriptions                                         | Keep in sync with this table.                                  |

### Target implementation notes (copy/paste guidance)
- Code quality: `fmt`, `clippy`, `cfix`, `fix`, `lint`
  - Use `--all-targets` for lint; prefer `-D warnings`.
  - `cfix`/`fix` should allow dirty trees to support active dev.
- Build: `check`, `build`, `buildr`, `qbuild`
  - `qbuild` must set `RUSTFLAGS="-Awarnings"` inline to avoid polluting env.
- Testing: `test`, `qtest`
  - `qtest` mirrors `qbuild` warning suppression.
- Maintenance: `clean`, `wip`
  - `wip` is root-only; never iterate it to avoid nested git commits.
- Discovery: `help`
  - Should render a concise target list; update when adding/removing targets.

## Development workflows

### Standard development cycle
```bash
make fmt
make cfix
make test
make build
```

### Rapid iteration
```bash
make qbuild
make qtest
```

### Code review prep
```bash
make fmt
make cfix
make clippy
make test
make build
```

### Work in progress
```bash
make wip
```

## Error and warnings handling

- Root uses `set -e` in iteration; the first failing workspace stops execution and surfaces its output.
- Quick targets (`qbuild`, `qtest`) set `RUSTFLAGS="-Awarnings"` to silence warnings.
- Standard targets use default warning behavior; clippy targets should deny warnings.

## Flag conventions

- Workspace-wide operations: `--workspace` where applicable.
- Crate-specific operations: `-p <crate_name>`.
- Linting: `--all-targets` (and optionally `--all-features`) to cover bins/tests/examples.

## Scalability

### Adding a new workspace
1) Add the folder to `WORKSPACES` in the root `Makefile`.
2) Add `Makefile` to the new workspace with all targets above.
3) Root targets automatically iterate the new workspace.

### Adding a new target
1) Add to `.PHONY` in all Makefiles.
2) Implement in each workspace/crate Makefile.
3) Add iteration logic in root `Makefile` (if not direct).
4) Update this specification and the `help` outputs.

## Usage examples

### From repository root
```bash
make build
make test
make fmt
make clippy
make clean
```

### From a workspace folder
```bash
cd shared && make build
cd shared/elemental && make test
```

### Combining targets
```bash
make fmt && make cfix && make test
make qbuild && make qtest
```

## Best practices

1) Run `fmt` before committing.
2) Use `cfix`/`clippy` to keep the tree warning-free.
3) Use quick targets for tight inner loops; standard targets for CI and reviews.
4) Keep `help` accurate; it is the first stop for new teammates.

## Future considerations

- Path-specific targets for larger monorepos.
- Optional parallel execution (`make -j`).
- Conditional target inclusion by workspace type.
- CI/CD integration for target mapping.
