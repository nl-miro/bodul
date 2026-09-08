FROM rust:1-slim-bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        cmake \
        libpq-dev \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

# Cargo registry config (defines the `margo` registry referenced by mulac-kernel's
# dependency metadata); required for the git dependency's manifests to load.
COPY .cargo .cargo

COPY Cargo.toml Cargo.lock ./
COPY apps/mvp/Cargo.toml apps/mvp/Cargo.toml
COPY lib/money/Cargo.toml lib/money/Cargo.toml
COPY lib/bodul_shared/Cargo.toml lib/bodul_shared/Cargo.toml
COPY apps apps
COPY lib lib

RUN cargo build --release -p mvp --bin server

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /workspace/target/release/server /usr/local/bin/bodul-server

ENV BODUL_SERVER_ADDR=0.0.0.0:3000
EXPOSE 3000

CMD ["/usr/local/bin/bodul-server"]
