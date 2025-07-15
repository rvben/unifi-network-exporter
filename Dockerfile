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

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r -g 1000 appuser && \
    useradd -r -u 1000 -g appuser appuser

# Copy the binary from builder
COPY --from=builder /app/target/release/unifi-network-exporter /usr/local/bin/unifi-network-exporter

# Switch to non-root user
USER appuser

# Expose metrics port
EXPOSE 9897

# Run the binary
ENTRYPOINT ["unifi-network-exporter"]