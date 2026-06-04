.PHONY: help security-checks clippy fmt asan tsan audit outdated test build clean

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

security-checks: ## Run all security and quality checks (required for deployment)
	@echo "🔒 Running security and quality checks for Health OS deployment..."
	@bash scripts/security-checks.sh

clippy: ## Run clippy with deny warnings
	@echo "Running clippy with deny warnings..."
	@cargo clippy --all-targets --all-features -- -D warnings -D clippy::all -D clippy::pedantic -A clippy::too_many_arguments -A clippy::missing_errors_doc

fmt: ## Check code formatting
	@echo "Checking code formatting..."
	@cargo fmt --all -- --check

fmt-fix: ## Fix code formatting
	@echo "Fixing code formatting..."
	@cargo fmt --all

asan: ## Run tests with AddressSanitizer (Linux only)
	@echo "Running tests with AddressSanitizer..."
	@if [ "$$(uname)" = "Linux" ]; then \
		export RUSTFLAGS="-Z sanitizer=address"; \
		export ASAN_OPTIONS="detect_leaks=1:halt_on_error=0"; \
		cargo test --all --all-features; \
	else \
		echo "AddressSanitizer is Linux-only"; \
	fi

tsan: ## Run tests with ThreadSanitizer (Linux only)
	@echo "Running tests with ThreadSanitizer..."
	@if [ "$$(uname)" = "Linux" ]; then \
		export RUSTFLAGS="-Z sanitizer=thread"; \
		cargo test --all --all-features; \
	else \
		echo "ThreadSanitizer is Linux-only"; \
	fi

audit: ## Run security audit on dependencies
	@echo "Running security audit..."
	@cargo audit

outdated: ## Check for outdated dependencies
	@echo "Checking for outdated dependencies..."
	@cargo outdated --exit-code 1 || true

test: ## Run all tests
	@echo "Running all tests..."
	@cargo test --all --all-features

build: ## Build all crates
	@echo "Building all crates..."
	@cargo build --all --all-features

build-release: ## Build all crates in release mode
	@echo "Building all crates in release mode..."
	@cargo build --all --all-features --release

clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	@cargo clean
