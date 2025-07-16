# Build stage
FROM rust:1.88 AS builder

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage - use debian slim for CA certificates support
FROM debian:12-slim

# Install CA certificates
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/unifi-network-exporter /usr/local/bin/unifi-network-exporter

# Expose metrics port
EXPOSE 9897

# Run the binary
ENTRYPOINT ["/usr/local/bin/unifi-network-exporter"]