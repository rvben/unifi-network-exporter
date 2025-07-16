# UniFi Network Exporter

[![CI](https://github.com/rvben/unifi-network-exporter/actions/workflows/ci.yml/badge.svg)](https://github.com/rvben/unifi-network-exporter/actions/workflows/ci.yml)
[![Release](https://github.com/rvben/unifi-network-exporter/actions/workflows/release.yml/badge.svg)](https://github.com/rvben/unifi-network-exporter/actions/workflows/release.yml)
[![Docker Pulls](https://img.shields.io/docker/pulls/rvben/unifi-network-exporter)](https://hub.docker.com/r/rvben/unifi-network-exporter)
[![Crates.io](https://img.shields.io/crates/v/unifi-network-exporter)](https://crates.io/crates/unifi-network-exporter)

Prometheus exporter for UniFi Network Controller metrics.

**Tested with UniFi Network Controller version 9.3.43**

## Features

- **Device Metrics**: Monitor UniFi devices (APs, switches, gateways)
  - Device information, uptime, and adoption status
  - CPU and memory usage
  - Network traffic (bytes/packets)
  
- **Client Metrics**: Track connected clients
  - Client information and connection details
  - WiFi signal strength
  - Bandwidth usage per client
  - Client counts by type, network, and guest status

- **Site Metrics**: Multi-site support
  - Total site count

## Quick Start

### Using Docker

```bash
docker run -d \
  -p 9897:9897 \
  -e UNIFI_CONTROLLER_URL=https://192.168.1.1:8443 \
  -e UNIFI_API_KEY=your-api-key \
  rvben/unifi-network-exporter:latest
```

### Using Docker Compose

```yaml
version: '3.8'

services:
  unifi-network-exporter:
    image: rvben/unifi-network-exporter:latest
    restart: unless-stopped
    ports:
      - "9897:9897"
    environment:
      - UNIFI_CONTROLLER_URL=https://192.168.1.1:8443
      - UNIFI_API_KEY=your-api-key
      - UNIFI_SITE=default
      - VERIFY_SSL=false
```

### Using Binary

```bash
# Download the latest release
wget https://github.com/rvben/unifi-network-exporter/releases/latest/download/unifi-network-exporter-x86_64-unknown-linux-gnu.tar.gz
tar -xzf unifi-network-exporter-x86_64-unknown-linux-gnu.tar.gz

# Run the exporter
UNIFI_CONTROLLER_URL=https://192.168.1.1:8443 \
UNIFI_API_KEY=your-api-key \
./unifi-network-exporter
```

## Configuration

The exporter can be configured via environment variables or command-line arguments:

| Environment Variable | CLI Flag | Default | Description |
|---------------------|----------|---------|-------------|
| `UNIFI_CONTROLLER_URL` | `--controller-url` | *required* | UniFi Controller URL (e.g., https://192.168.1.1:8443) |
| `UNIFI_API_KEY` | `--api-key` | *optional* | UniFi API key (recommended) |
| `UNIFI_USERNAME` | `--username` | *optional* | UniFi username (if no API key) |
| `UNIFI_PASSWORD` | `--password` | *optional* | UniFi password (if no API key) |
| `UNIFI_SITE` | `--site` | `default` | UniFi site name |
| `METRICS_PORT` | `--port` | `9897` | Port to expose metrics on |
| `POLL_INTERVAL` | `--poll-interval` | `30` | Poll interval in seconds |
| `LOG_LEVEL` | `--log-level` | `info` | Log level (trace, debug, info, warn, error) |
| `HTTP_TIMEOUT` | `--http-timeout` | `10` | HTTP timeout in seconds |
| `VERIFY_SSL` | `--verify-ssl` | `true` | Verify SSL certificates |

## Metrics

### Device Metrics

- `unifi_device_info` - Device information (labels: id, name, mac, type, model, version)
- `unifi_device_uptime_seconds` - Device uptime in seconds
- `unifi_device_adopted` - Device adoption status (1=adopted, 0=not adopted)
- `unifi_device_state` - Device state
- `unifi_device_cpu_usage` - Device CPU usage (load average)
- `unifi_device_memory_usage_ratio` - Device memory usage ratio
- `unifi_device_memory_total_bytes` - Device total memory in bytes
- `unifi_device_bytes_total` - Total bytes transferred (labels: direction)
- `unifi_device_packets_total` - Total packets transferred (labels: direction)

### Client Metrics

- `unifi_client_info` - Client information (labels: id, mac, hostname, name, ip, network, ap_mac)
- `unifi_client_bytes_total` - Total bytes transferred by client (labels: direction)
- `unifi_client_signal_strength_dbm` - Client WiFi signal strength in dBm
- `unifi_client_uptime_seconds` - Client connection uptime in seconds
- `unifi_clients_total` - Total number of clients (labels: type, network, is_guest)

### Site Metrics

- `unifi_sites_total` - Total number of sites

## Prometheus Configuration

Add this to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'unifi'
    static_configs:
      - targets: ['localhost:9897']
```

## Building from Source

### Prerequisites

- Rust 1.88 or later
- Make (optional)

### Build

```bash
# Clone the repository
git clone https://github.com/rvben/unifi-network-exporter.git
cd unifi-network-exporter

# Build release binary
make release

# Or using cargo directly
cargo build --release
```

### Run Tests

```bash
make test
```

## Security Considerations

- **Credentials**: Store UniFi credentials securely (use secrets management in production)
- **SSL Verification**: Only disable SSL verification (`VERIFY_SSL=false`) for self-signed certificates
- **Network Access**: The exporter needs network access to your UniFi Controller
- **Metrics Exposure**: Consider restricting access to the metrics endpoint

## Development

### Local Development

```bash
# Run with cargo
cargo run

# Run with make
make run

# Run tests
make test

# Check code quality
make check
```

### Docker Development

```bash
# Build Docker image
make docker-build

# Run Docker container
make docker-run
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Uses [Prometheus](https://prometheus.io/) client library
- Inspired by other UniFi exporters