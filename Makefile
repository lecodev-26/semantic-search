.PHONY: build test bench clean run install format lint docs release

# Default target
all: build

# Build debug version
build:
cargo build

# Build release version
release:
cargo build --release

# Run tests
test:
cargo test --all-features

# Run benchmarks
bench:
cargo bench --all-features

# Format code
format:
cargo fmt --all

# Lint
lint:
cargo clippy --all-targets --all-features -- -D warnings

# Generate documentation
docs:
cargo doc --no-deps

# Clean build artifacts
clean:
cargo clean
rm -rf .semantic-search/
rm -rf *.lmdb/

# Install locally
install: release
cargo install --path .

# Run with example
example:
cargo run -- index ./examples/test-project
cargo run -- search "function that computes fibonacci"

# Build for all platforms
cross-build:
cargo build --release --target aarch64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-musl
cargo build --release --target x86_64-pc-windows-gnu

# Run in docker
docker-build:
docker build -t semantic-search .

docker-run:
docker run -it --rm semantic-search

# Help
help:
@echo "Available commands:"
@echo "  make build        - Build debug version"
@echo "  make release      - Build release version"
@echo "  make test         - Run tests"
@echo "  make bench        - Run benchmarks"
@echo "  make format       - Format code"
@echo "  make lint         - Run linter"
@echo "  make docs         - Generate documentation"
@echo "  make clean        - Clean build artifacts"
@echo "  make install      - Install locally"
@echo "  make example      - Run example workflow"
@echo "  make cross-build  - Build for multiple platforms"
