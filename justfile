# Cyrup Sugars - Development Commands
# Run examples and tests with the correct feature combinations

# Default recipe - shows available commands
default:
    @just --list

# Clean and format the codebase
fmt:
    cargo fmt

# Check compilation with all warnings as errors
check:
    cargo fmt && cargo check --message-format short --quiet

# Run all tests with nextest
test:
    cargo nextest run

# Run specific feature tests
test-collections:
    cargo nextest run --features collections

test-async:
    cargo nextest run --features tokio-async

test-macros:
    cargo nextest run --features macros

test-hashbrown-json:
    cargo nextest run --features hashbrown-json

test-gix:
    cargo nextest run --features gix-interop

# Run all tests with all features
test-all:
    cargo nextest run --features full

# Comprehensive feature testing - test every feature in isolation and combination
test-features:
    @echo "🧪 Testing all feature combinations..."
    @echo "📦 Installing nextest if needed..."
    @cargo install cargo-nextest --locked 2>/dev/null || true
    @echo "🔧 Testing minimal (no features)..."
    cargo nextest run --no-default-features
    @echo "🔧 Testing std only..."
    cargo nextest run --no-default-features --features std
    @echo "🔧 Testing collections only..."
    cargo nextest run --no-default-features --features collections
    @echo "🔧 Testing async only..."
    cargo nextest run --no-default-features --features async
    @echo "🔧 Testing macros only..."
    cargo nextest run --no-default-features --features macros
    @echo "🔧 Testing tokio-async backend..."
    cargo nextest run --no-default-features --features tokio-async
    @echo "🔧 Testing std-async backend..."
    cargo nextest run --no-default-features --features std-async
    @echo "🔧 Testing crossbeam-async backend..."
    cargo nextest run --no-default-features --features crossbeam-async
    @echo "🔧 Testing serde support..."
    cargo nextest run --no-default-features --features collections,serde
    @echo "🔧 Testing hashbrown-json..."
    cargo nextest run --no-default-features --features hashbrown-json
    @echo "🔧 Testing gix-interop..."
    cargo nextest run --no-default-features --features gix-interop
    @echo "🔧 Testing async+collections..."
    cargo nextest run --no-default-features --features async,collections
    @echo "🔧 Testing macros+collections..."
    cargo nextest run --no-default-features --features macros,collections
    @echo "🔧 Testing all async backends..."
    cargo nextest run --no-default-features --features tokio-async,std-async,crossbeam-async
    @echo "🔧 Testing full feature set..."
    cargo nextest run --features full
    @echo "🎉 All feature combinations tested successfully!"

# Test specific feature combinations
test-minimal:
    cargo nextest run --no-default-features

test-std:
    cargo nextest run --no-default-features --features std

test-collections-serde:
    cargo nextest run --no-default-features --features collections,serde

test-async-backends:
    cargo nextest run --no-default-features --features tokio-async,std-async,crossbeam-async

test-integration:
    cargo nextest run --no-default-features --features async,collections,macros

# Run examples with required features
examples: example-collections example-async example-ai-agent example-api-config example-showcase

# Basic collections example (requires collections feature)
example-collections:
    cargo run --example collections_basic --features collections

# Async task pipeline example (requires tokio-async feature)
example-async:
    cargo run --example async_task_pipeline --features tokio-async

# AI agent builder example (requires hashbrown-json feature)
example-ai-agent:
    cargo run --example ai_agent_builder --features hashbrown-json

# API config builder example (requires hashbrown-json feature)  
example-api-config:
    cargo run --example api_config_builder --features hashbrown-json

# Complete showcase example (requires all features)
example-showcase:
    cargo run --example complete_showcase --features full

# Check that all examples compile with correct features
check-examples:
    @echo "Checking collections example..."
    cargo check --example collections_basic --features collections
    @echo "Checking async example..."
    cargo check --example async_task_pipeline --features tokio-async
    @echo "Checking AI agent example..."
    cargo check --example ai_agent_builder --features hashbrown-json
    @echo "Checking API config example..."
    cargo check --example api_config_builder --features hashbrown-json
    @echo "Checking showcase example..."
    cargo check --example complete_showcase --features full
    @echo "✅ All examples compile successfully!"

# Production readiness check - zero warnings and all tests pass
production-ready:
    @echo "🔍 Running production readiness checks..."
    cargo fmt
    cargo check --message-format short --quiet
    cargo clippy --all-features -- -D warnings
    cargo nextest run --features full
    @just check-examples
    @echo "🚀 Production ready! Zero warnings, all tests pass, all examples compile."

# Quick development iteration
dev:
    cargo fmt && cargo check --message-format short --quiet && cargo nextest run --features tokio-async

# Run clippy with all features and fail on warnings
clippy:
    cargo clippy --all-features -- -D warnings

# Build documentation
docs:
    cargo doc --all-features --open

# Clean build artifacts
clean:
    cargo clean
