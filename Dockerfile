# Build stage
FROM rust:1-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy src to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy actual source
COPY src ./src
COPY templates ./templates

# Build the real binary (touch to update mtime)
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:trixie-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/rhxn ./

# Copy static assets
COPY assets ./assets

ENV HOST=0.0.0.0
ENV PORT=3000
EXPOSE 3000

CMD ["./rhxn"]
