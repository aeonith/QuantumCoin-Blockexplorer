FROM rust:1.80 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
ENV RUST_LOG=info
WORKDIR /app
COPY --from=builder /app/target/release/qtc-indexer /usr/local/bin/qtc-indexer
CMD ["qtc-indexer"]
