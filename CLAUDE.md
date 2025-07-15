# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

This repository contains a Prometheus exporter for UniFi Network Controller that monitors:
- Network devices (APs, switches, gateways)
- Connected clients
- Network health metrics

## Build and Development Commands

All commands are wrapped in a Makefile following a local-first CI/CD philosophy:

```bash
make build        # Build debug binary
make release      # Build release binary
make run          # Run exporter (requires UNIFI_* env vars)
make test         # Run tests (cargo test --verbose)
make lint         # Run Clippy linter (cargo clippy -- -D warnings)
make fmt          # Format code (cargo fmt)
make check        # Run format check + lint
make docker-build # Build Docker image
make docker-run   # Run Docker container (requires UNIFI_* env vars)
```

## Architecture

The exporter follows a modular structure:
- `config.rs` - CLI/environment configuration using clap
- `unifi.rs` - Async HTTP client for UniFi API with authentication handling
- `metrics.rs` - Prometheus metrics management using prometheus crate
- `main.rs` - Tokio async runtime, Axum web server, polling orchestration

Key patterns:
- Async/await throughout with Tokio runtime
- Axum for the metrics HTTP endpoint
- Cookie-based authentication with automatic re-authentication
- Structured logging with tracing
- Error handling: anyhow for application errors, thiserror for custom errors
- Non-blocking architecture with separate polling and HTTP serving tasks

## UniFi API Integration

The UniFi Controller API requires:
1. Cookie-based authentication via POST to `/api/login`
2. All subsequent requests must include authentication cookies
3. Automatic re-authentication on 401 responses
4. Support for self-signed certificates (configurable)

## Testing

Run tests with coverage:
```bash
make coverage
```

Test a single module:
```bash
cargo test unifi::
```

## Configuration

Required environment variables:
- `UNIFI_CONTROLLER_URL` - Controller URL (e.g., https://192.168.1.1:8443)
- `UNIFI_USERNAME` - Admin username
- `UNIFI_PASSWORD` - Admin password

Optional:
- `UNIFI_SITE` - Site name (default: "default")
- `VERIFY_SSL` - Verify SSL certificates (default: true)
- `METRICS_PORT` - Prometheus metrics port (default: 9897)
- `POLL_INTERVAL` - API polling interval in seconds (default: 30)

## Metrics Structure

Three main metric categories:
1. **Device Metrics** - Hardware stats, uptime, network traffic
2. **Client Metrics** - Connection info, signal strength, bandwidth
3. **Site Metrics** - Multi-site support

All metrics follow Prometheus naming conventions with appropriate labels.

## Security Considerations

1. Credentials are passed via environment variables
2. SSL verification can be disabled for self-signed certificates
3. Cookie-based authentication is handled automatically
4. No credentials are logged or exposed in metrics