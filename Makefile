# Makefile for ichimi-server

# Variables
BINARY_NAME = ichimi
INSTALL_DIR = $(HOME)/.local/bin
TARGET_DIR = target
RELEASE_BIN = $(TARGET_DIR)/release/$(BINARY_NAME)
DEBUG_BIN = $(TARGET_DIR)/debug/$(BINARY_NAME)

# Default target
.DEFAULT_GOAL := help

# Phony targets
.PHONY: help build build-release install install-debug clean test check run run-web

# Help target
help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

# Build targets
build: ## Build debug version (fast)
	@echo "Building debug version..."
	@cargo build
	@echo "✅ Debug build complete: $(DEBUG_BIN)"

build-release: ## Build release version (optimized)
	@echo "Building release version (this may take a while)..."
	@cargo build --release
	@echo "✅ Release build complete: $(RELEASE_BIN)"

# Install targets
install: build-release ## Build and install release version to ~/.local/bin
	@echo "Installing to $(INSTALL_DIR)..."
	@mkdir -p $(INSTALL_DIR)
	@cp $(RELEASE_BIN) $(INSTALL_DIR)/$(BINARY_NAME)
	@chmod +x $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "✅ Installed to $(INSTALL_DIR)/$(BINARY_NAME)"
	@$(INSTALL_DIR)/$(BINARY_NAME) --version

install-debug: build ## Build and install debug version to ~/.local/bin
	@echo "Installing debug build to $(INSTALL_DIR)..."
	@mkdir -p $(INSTALL_DIR)
	@cp $(DEBUG_BIN) $(INSTALL_DIR)/$(BINARY_NAME)
	@chmod +x $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "✅ Installed debug build to $(INSTALL_DIR)/$(BINARY_NAME)"
	@$(INSTALL_DIR)/$(BINARY_NAME) --version

# Development targets
test: ## Run all tests
	@echo "Running tests..."
	@cargo test

check: ## Check code without building
	@echo "Checking code..."
	@cargo check

clippy: ## Run clippy linter
	@echo "Running clippy..."
	@cargo clippy -- -D warnings

fmt: ## Format code
	@echo "Formatting code..."
	@cargo fmt

fmt-check: ## Check code formatting
	@echo "Checking code formatting..."
	@cargo fmt -- --check

# Run targets
run: ## Run debug version
	@cargo run --bin $(BINARY_NAME)

run-web: ## Run debug version with web interface
	@cargo run --bin $(BINARY_NAME) -- --web

run-release: ## Run release version
	@cargo run --release --bin $(BINARY_NAME)

run-release-web: ## Run release version with web interface
	@cargo run --release --bin $(BINARY_NAME) -- --web

# Clean target
clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	@cargo clean
	@echo "✅ Clean complete"

# Quick install for development
dev-install: build ## Quick install for development (debug build)
	@./install-local.sh debug

# Quick install for production
prod-install: ## Production install (release build)
	@./install-local.sh release

# Uninstall
uninstall: ## Remove ichimi from ~/.local/bin
	@if [ -f "$(INSTALL_DIR)/$(BINARY_NAME)" ]; then \
		rm -f "$(INSTALL_DIR)/$(BINARY_NAME)"; \
		echo "✅ Uninstalled $(BINARY_NAME) from $(INSTALL_DIR)"; \
	else \
		echo "⚠️  $(BINARY_NAME) not found in $(INSTALL_DIR)"; \
	fi

# Version management
version: ## Show current version
	@grep '^version' Cargo.toml | head -1 | cut -d'"' -f2

# CI/CD helpers
ci-test: fmt-check clippy test ## Run all CI checks

# Install from GitHub (latest release)
install-latest: ## Install latest release from GitHub
	cargo install --git https://github.com/chronista-club/ichimi-server

# Benchmark (if you add benchmarks later)
bench: ## Run benchmarks
	@cargo bench