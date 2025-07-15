# Build stage
FROM rust:1.88-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM alpine:3.21

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Create non-root user
RUN addgroup -g 1000 -S appuser && \
    adduser -u 1000 -S appuser -G appuser

# Copy the binary from builder
COPY --from=builder /app/target/release/unifi-network-exporter /usr/local/bin/unifi-network-exporter

# Switch to non-root user
USER appuser

# Expose metrics port
EXPOSE 9897

# Run the binary
ENTRYPOINT ["unifi-network-exporter"]