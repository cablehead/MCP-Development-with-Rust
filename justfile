# Justfile for MCP Rust Examples
#
# This file contains convenient commands for common development tasks.
# Just is a command runner that provides a simple way to define and run
# project-specific commands. Install with: cargo install just
#
# Usage: just <command>
# To see all available commands: just --list

# Set shell for all recipes
set shell := ["bash", "-c"]

# Load environment variables from .env file if it exists
set dotenv-load := true

# Default recipe - shows help
default:
    @just --list

# 🏗️  BUILD COMMANDS
# ==================

# Build all examples in debug mode
build:
    @echo "🔨 Building all examples..."
    cargo build --all-targets

# Build all examples in release mode
build-release:
    @echo "🔨 Building all examples in release mode..."
    cargo build --all-targets --release

# Build a specific example
build-example example:
    @echo "🔨 Building example: {{example}}"
    cargo build --bin {{example}}

# Clean build artifacts
clean:
    @echo "🧹 Cleaning build artifacts..."
    cargo clean

# 🧪 TEST COMMANDS
# ================

# Run all tests
test:
    @echo "🧪 Running all tests..."
    cargo test --all-targets --verbose

# Run tests with coverage
test-coverage:
    @echo "🧪 Running tests with coverage..."
    cargo install cargo-tarpaulin --locked
    cargo tarpaulin --all-targets --out Html --output-dir target/coverage

# Run tests for a specific example
test-example example:
    @echo "🧪 Testing example: {{example}}"
    cargo test --bin {{example}}

# Run integration tests only
test-integration:
    @echo "🧪 Running integration tests..."
    cargo test --test '*'

# Run unit tests only
test-unit:
    @echo "🧪 Running unit tests..."
    cargo test --lib

# 🚀 RUN COMMANDS
# ===============

# List all available examples
list-examples:
    @echo "📋 Available examples:"
    @cargo metadata --format-version 1 | jq -r '.packages[0].targets[] | select(.kind[] == "bin") | "  - " + .name'

# Run a specific example
run example *args:
    @echo "🚀 Running example: {{example}}"
    cargo run --bin {{example}} {{args}}

# Run a specific example in release mode
run-release example *args:
    @echo "🚀 Running example in release mode: {{example}}"
    cargo run --release --bin {{example}} {{args}}

# Run all examples sequentially (for demo purposes)
run-all:
    @echo "🚀 Running all examples..."
    @for example in $(cargo metadata --format-version 1 | jq -r '.packages[0].targets[] | select(.kind[] == "bin") | .name'); do \
        echo "Running $example..."; \
        timeout 10s cargo run --bin $example || echo "Example $example completed or timed out"; \
    done

# 🎨 CODE QUALITY COMMANDS
# ========================

# Format code with rustfmt
fmt:
    @echo "🎨 Formatting code..."
    cargo fmt --all

# Check code formatting
fmt-check:
    @echo "🎨 Checking code formatting..."
    cargo fmt --all -- --check

# Run clippy linter
lint:
    @echo "🔍 Running clippy linter..."
    cargo clippy --all-targets --all-features -- -D warnings

# Run clippy with fixes
lint-fix:
    @echo "🔧 Running clippy with automatic fixes..."
    cargo clippy --all-targets --all-features --fix --allow-dirty

# Run all quality checks
quality: fmt-check lint
    @echo "✅ All quality checks passed!"

# 📚 DOCUMENTATION COMMANDS
# =========================

# Build documentation
docs:
    @echo "📚 Building documentation..."
    cargo doc --all --no-deps --document-private-items

# Build and open documentation
docs-open:
    @echo "📚 Building and opening documentation..."
    cargo doc --all --no-deps --document-private-items --open

# Check documentation
docs-check:
    @echo "📚 Checking documentation..."
    RUSTDOCFLAGS="-D warnings" cargo doc --all --no-deps --document-private-items

# 🔒 SECURITY COMMANDS
# ====================

# Run security audit
audit:
    @echo "🔒 Running security audit..."
    cargo audit

# Check for yanked dependencies
audit-yanked:
    @echo "🔒 Checking for yanked dependencies..."
    cargo audit --deny yanked

# Install or update cargo-audit
install-audit:
    @echo "🔒 Installing/updating cargo-audit..."
    cargo install cargo-audit --locked

# 📦 DEPENDENCY COMMANDS
# ======================

# Update dependencies
update:
    @echo "📦 Updating dependencies..."
    cargo update

# Check for outdated dependencies
outdated:
    @echo "📦 Checking for outdated dependencies..."
    cargo install cargo-outdated --locked
    cargo outdated

# Show dependency tree
tree:
    @echo "📦 Showing dependency tree..."
    cargo tree

# Check for unused dependencies
unused-deps:
    @echo "📦 Checking for unused dependencies..."
    cargo +nightly udeps --all-targets

# Install development tools
install-tools:
    @echo "🛠️ Installing development tools..."
    cargo install cargo-audit --locked
    cargo install cargo-outdated --locked
    cargo install cargo-tarpaulin --locked
    cargo install cargo-watch --locked
    rustup component add rustfmt clippy

# 🔄 WATCH COMMANDS
# =================

# Watch and rebuild on file changes
watch:
    @echo "👀 Watching for file changes..."
    cargo install cargo-watch --locked
    cargo watch -x "build --all-targets"

# Watch and run tests on file changes
watch-test:
    @echo "👀 Watching and running tests..."
    cargo install cargo-watch --locked
    cargo watch -x "test --all-targets"

# Watch and run a specific example on changes
watch-example example:
    @echo "👀 Watching and running example: {{example}}"
    cargo install cargo-watch --locked
    cargo watch -x "run --bin {{example}}"

# 🚦 CI/CD COMMANDS
# =================

# Run all CI checks locally
ci: fmt-check lint test docs-check audit
    @echo "✅ All CI checks passed!"

# Simulate release build
ci-release: build-release test audit docs-check
    @echo "✅ Release simulation complete!"

# Run security checks
ci-security: audit audit-yanked
    @echo "✅ Security checks complete!"

# 🎯 QUICK EXAMPLES
# =================

# Quick demo of the hello world example
demo-hello:
    @echo "🎯 Running Hello World demo..."
    cargo run --bin example_01_hello_world

# Quick demo of the calculator example
demo-calc:
    @echo "🎯 Running Calculator demo..."
    cargo run --bin example_02_calculator

# Quick demo of the task queue example
demo-queue:
    @echo "🎯 Running Task Queue demo..."
    cargo run --bin example_12_task_queue

# 🏷️ RELEASE COMMANDS
# ===================

# Create a new version tag (requires version argument)
tag version:
    @echo "🏷️ Creating version tag: v{{version}}"
    git tag -a "v{{version}}" -m "Release version {{version}}"
    @echo "Push with: git push origin v{{version}}"

# Prepare for release
prepare-release version: ci
    @echo "🚀 Preparing release {{version}}..."
    @echo "1. Update version in Cargo.toml"
    @echo "2. Update CHANGELOG.md"
    @echo "3. Commit changes"
    @echo "4. Run: just tag {{version}}"
    @echo "5. Push tag to trigger release workflow"

# 🧽 CLEANUP COMMANDS
# ===================

# Clean all build artifacts and caches
clean-all: clean
    @echo "🧽 Cleaning all artifacts and caches..."
    rm -rf target/
    rm -rf Cargo.lock
    cargo clean

# Reset to clean state
reset: clean-all
    @echo "🔄 Resetting to clean state..."
    git clean -fd
    git reset --hard HEAD

# 📊 UTILITY COMMANDS
# ===================

# Show project information
info:
    @echo "📊 Project Information:"
    @echo "Name: MCP Rust Examples"
    @echo "Version: $(cargo metadata --format-version 1 | jq -r '.packages[0].version')"
    @echo "Dependencies: $(cargo metadata --format-version 1 | jq '.packages[0].dependencies | length')"
    @echo "Examples: $(cargo metadata --format-version 1 | jq '.packages[0].targets | map(select(.kind[] == "bin")) | length')"
    @echo "Rust version: $(rustc --version)"

# Check environment and dependencies
doctor:
    @echo "🏥 Environment Check:"
    @echo "Rust: $(rustc --version)"
    @echo "Cargo: $(cargo --version)"
    @echo "Just: $(just --version)"
    @command -v jq >/dev/null 2>&1 && echo "jq: $(jq --version)" || echo "jq: Not installed (recommended for JSON processing)"
    @command -v git >/dev/null 2>&1 && echo "Git: $(git --version)" || echo "Git: Not installed"

# Show disk usage of build artifacts
disk-usage:
    @echo "💽 Disk Usage:"
    @du -sh target/ 2>/dev/null || echo "No target directory found"
    @du -sh ~/.cargo/registry/ 2>/dev/null || echo "No cargo registry cache found"

# 🔧 DEVELOPMENT HELPERS
# ======================

# Generate a new example template
new-example name:
    @echo "🔧 Creating new example: {{name}}"
    @mkdir -p src/examples
    @echo '// Example: {{name}}' > src/examples/{{name}}.rs
    @echo '//' >> src/examples/{{name}}.rs
    @echo '// This example demonstrates [describe what this example shows].' >> src/examples/{{name}}.rs
    @echo '' >> src/examples/{{name}}.rs
    @echo 'use tracing::{info, error};' >> src/examples/{{name}}.rs
    @echo '' >> src/examples/{{name}}.rs
    @echo '// Function: main' >> src/examples/{{name}}.rs
    @echo '//' >> src/examples/{{name}}.rs
    @echo '// This is the entry point of the program.' >> src/examples/{{name}}.rs
    @echo '// [Add description of what this example does]' >> src/examples/{{name}}.rs
    @echo '#[tokio::main]' >> src/examples/{{name}}.rs
    @echo 'async fn main() -> Result<(), Box<dyn std::error::Error>> {' >> src/examples/{{name}}.rs
    @echo '    // Initialize the tracing subscriber for logging' >> src/examples/{{name}}.rs
    @echo '    tracing_subscriber::fmt()' >> src/examples/{{name}}.rs
    @echo '        .with_env_filter("info")' >> src/examples/{{name}}.rs
    @echo '        .init();' >> src/examples/{{name}}.rs
    @echo '' >> src/examples/{{name}}.rs
    @echo '    info!("Starting {{name}} example");' >> src/examples/{{name}}.rs
    @echo '' >> src/examples/{{name}}.rs
    @echo '    // Add your example code here' >> src/examples/{{name}}.rs
    @echo '' >> src/examples/{{name}}.rs
    @echo '    info!("{{name}} example completed successfully");' >> src/examples/{{name}}.rs
    @echo '    Ok(())' >> src/examples/{{name}}.rs
    @echo '}' >> src/examples/{{name}}.rs
    @echo "[[bin]]" >> Cargo.toml
    @echo "name = \"{{name}}\"" >> Cargo.toml
    @echo "path = \"src/examples/{{name}}.rs\"" >> Cargo.toml
    @echo "✅ Example {{name}} created!"

# Run a comprehensive development check
dev-check: fmt lint test docs-check audit
    @echo "🔍 Development check complete!"
    @echo "Your code is ready for commit! 🎉" 