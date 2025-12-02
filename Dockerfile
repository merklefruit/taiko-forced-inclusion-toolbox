FROM rust:1.88-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 appuser

WORKDIR /app

COPY --from=builder /app/target/release/taiko-forced-inclusion-toolbox /app/taiko-forced-inclusion-toolbox

RUN chown appuser:appuser /app/taiko-forced-inclusion-toolbox

USER appuser

ENTRYPOINT ["/app/taiko-forced-inclusion-toolbox"]
