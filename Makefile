# Makefile for UniFi Network Exporter

# Variables
BINARY_NAME=unifi-network-exporter
DOCKER_IMAGE=unifi-network-exporter
DOCKER_TAG=latest

# Default target
.PHONY: all
all: build

# Build the binary
.PHONY: build
build:
	cargo build

# Build release binary
.PHONY: release
release:
	cargo build --release

# Run the exporter
.PHONY: run
run:
	cargo run

# Run tests
.PHONY: test
test:
	cargo test --verbose

# Run linter
.PHONY: lint
lint:
	cargo clippy -- -D warnings

# Format code
.PHONY: fmt
fmt:
	cargo fmt

# Check formatting
.PHONY: fmt-check
fmt-check:
	cargo fmt -- --check

# Run all checks (format + lint)
.PHONY: check
check: fmt-check lint

# Build Docker image
.PHONY: docker-build
docker-build:
	docker build -t $(DOCKER_IMAGE):$(DOCKER_TAG) .

# Run Docker container
.PHONY: docker-run
docker-run:
	docker run --rm -p 9897:9897 \
		-e UNIFI_CONTROLLER_URL=$(UNIFI_CONTROLLER_URL) \
		-e UNIFI_USERNAME=$(UNIFI_USERNAME) \
		-e UNIFI_PASSWORD=$(UNIFI_PASSWORD) \
		$(DOCKER_IMAGE):$(DOCKER_TAG)

# Clean build artifacts
.PHONY: clean
clean:
	cargo clean

# Run code coverage
.PHONY: coverage
coverage:
	cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out html

# Help
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  build         - Build debug binary"
	@echo "  release       - Build release binary"
	@echo "  run           - Run the exporter"
	@echo "  test          - Run tests"
	@echo "  lint          - Run clippy linter"
	@echo "  fmt           - Format code"
	@echo "  fmt-check     - Check code formatting"
	@echo "  check         - Run format check and linter"
	@echo "  docker-build  - Build Docker image"
	@echo "  docker-run    - Run Docker container"
	@echo "  clean         - Clean build artifacts"
	@echo "  coverage      - Generate code coverage report"
	@echo "  help          - Show this help message"