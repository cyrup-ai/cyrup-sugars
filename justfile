# Sugars project justfile

# Default recipe - show available commands
default:
    @just --list

# Run all checks: cargo check, tests, and examples
check:
    @echo "Running cargo check..."
    cargo check
    @echo ""
    @echo "Running tests with nextest..."
    cargo nextest run
    @echo ""
    @echo "Running all examples..."
    @echo "----------------------------------------"
    @echo "Running array_tuple_syntax example..."
    cd examples/array_tuple_syntax && cargo run
    @echo ""
    @echo "----------------------------------------"
    @echo "Running async_task_example..."
    cd examples/async_task_example && cargo run
    @echo ""
    @echo "----------------------------------------"
    @echo "Running one_or_many_example..."
    cd examples/one_or_many_example && cargo run
    @echo ""
    @echo "----------------------------------------"
    @echo "Running zero_one_or_many_example..."
    cd examples/zero_one_or_many_example && cargo run
    @echo ""
    @echo "✅ All checks passed!"

# Build the project
build:
    cargo build

# Run tests
test:
    cargo nextest run

# Run a specific example
example name:
    cd examples/{{name}} && cargo run

# Clean build artifacts
clean:
    cargo clean

# Format code
fmt:
    cargo fmt --all

# Run clippy
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Get current version from workspace
get-version:
    @grep "^version" Cargo.toml | head -1 | cut -d'"' -f2

# Check release readiness
release-checklist:
    @echo "📋 Release Checklist"
    @echo "==================="
    @echo ""
    # Check git status
    @git diff --quiet || (echo "❌ Uncommitted changes detected" && exit 1)
    @git diff --cached --quiet || (echo "❌ Staged changes detected" && exit 1) 
    @echo "✅ Working directory clean"
    # Run tests
    @echo "🧪 Running tests..."
    @cargo test --all-features --quiet
    @echo "✅ All tests pass"
    @cargo nextest run --quiet
    @echo "✅ Nextest passes"
    # Check docs
    @echo "📚 Building documentation..."
    @cargo doc --all-features --no-deps --quiet
    @echo "✅ Documentation builds"
    # Check clippy
    @echo "📎 Running clippy..."
    @cargo clippy --all-targets --all-features --quiet -- -D warnings
    @echo "✅ Clippy clean"
    # Check formatting
    @echo "🎨 Checking formatting..."
    @cargo fmt --all -- --check
    @echo "✅ Code is formatted"
    # Check examples
    @echo "🔧 Checking examples..."
    @cargo build --examples --quiet
    @echo "✅ All examples compile"
    @echo ""
    @echo "📦 Current version: {{get-version}}"
    @echo ""
    @echo "✅ Ready for release!"

# Build the release tool if needed
build-release-tool:
    @cd tools/release && cargo build --release --quiet

# Set new version in workspace
set-version VERSION: build-release-tool
    @./tools/release/target/release/release-tool set {{VERSION}}

# Bump version (major, minor, or patch)
bump TYPE="patch": build-release-tool
    @./tools/release/target/release/release-tool bump {{TYPE}}

# Publish a single package
publish-package PACKAGE DRY="false":
    @echo "📦 Publishing {{PACKAGE}}..."
    @if [ "{{DRY}}" = "true" ]; then \
        cargo publish --package {{PACKAGE}} --dry-run; \
    else \
        cargo publish --package {{PACKAGE}}; \
    fi

# Wait for crates.io to index
wait-for-index:
    @echo "⏳ Waiting 15 seconds for crates.io to index..."
    @sleep 15

# Release all packages in dependency order
release TYPE="patch":
    # Check if ready
    just release-checklist
    # Bump version
    @echo "Bumping {{TYPE}} version..."
    just bump {{TYPE}}
    # Get new version
    @echo "New version: $(just get-version)"
    # Update lock file
    cargo update --workspace
    # Commit
    git add -A
    git commit -m "chore: release v{{get-version}}"
    # Tag
    git tag -a "v{{get-version}}" -m "Release v{{get-version}}"
    @echo "🚀 Starting release of v{{get-version}}"
    # Tier 0: no dependencies
    @echo "═══ Tier 0: Base packages ═══"
    just publish-package sugars_macros false
    just wait-for-index
    just publish-package sugars_collections false
    just wait-for-index
    just publish-package sugars_gix false
    just wait-for-index
    # Tier 1: depends on tier 0
    @echo "═══ Tier 1: First level dependencies ═══"
    just publish-package sugars_async_task false
    just wait-for-index
    # Tier 2: depends on tier 0 and 1
    @echo "═══ Tier 2: Second level dependencies ═══"
    just publish-package sugars_async_stream false
    just wait-for-index
    just publish-package sugars_builders false
    just wait-for-index
    just publish-package sugars_llm false
    just wait-for-index
    # Tier 3: main package
    @echo "═══ Tier 3: Main package ═══"
    just publish-package cyrup_sugars false
    # Push to git
    @echo "📤 Pushing to git..."
    git push origin main
    git push origin "v{{get-version}}"
    @echo "✅ Release v{{get-version}} complete!"

# Dry run release (no actual publishing)
release-dry TYPE="patch":
    # Check if ready
    just release-checklist
    @echo "🎭 DRY RUN - No actual publishing"
    # Show what would happen
    @echo "Would bump {{TYPE}} version"
    @echo "Current version: {{get-version}}"
    # Check each package
    @echo "═══ Checking packages ═══"
    just publish-package sugars_macros true
    just publish-package sugars_collections true
    just publish-package sugars_gix true
    just publish-package sugars_async_task true
    just publish-package sugars_async_stream true
    just publish-package sugars_builders true
    just publish-package sugars_llm true
    just publish-package cyrup_sugars true
    @echo "✅ Dry run complete"