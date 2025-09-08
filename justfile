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
    # Check git status (just warn, don't fail)
    @git diff --quiet && git diff --cached --quiet && echo "✅ Working directory clean" || echo "⚠️  Uncommitted changes will be included in release"
    # Run tests
    @echo "🧪 Running tests..."
    @cargo test --all-features --quiet
    @echo "✅ All tests pass"
    @cargo nextest run 2>/dev/null
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
    @echo "📦 Current version: $(just get-version)"
    @echo ""
    @echo "✅ Ready for release!"

# Manually update version in workspace Cargo.toml (no dependencies needed)
set-version-manual VERSION:
    @echo "Setting version to {{VERSION}}"
    @perl -i -pe 's/^version = ".*"/version = "{{VERSION}}"/' Cargo.toml
    @echo "Version set to {{VERSION}}"

# Calculate next version
next-version TYPE="patch":
    @perl -e '\
        $$v = "'"$(just get-version)"'"; \
        ($$major, $$minor, $$patch) = split /\./, $$v; \
        if ("{{TYPE}}" eq "major") { print int($$major+1).".0.0\n" } \
        elsif ("{{TYPE}}" eq "minor") { print "$$major.".int($$minor+1).".0\n" } \
        else { print "$$major.$$minor.".int($$patch+1)."\n" }'

# Bump version manually (no dependencies needed)
bump-manual TYPE="patch":
    @echo "Current version: $(just get-version)"
    @just set-version-manual $(just next-version {{TYPE}})
    
# Set new version in workspace (requires cargo-edit)
set-version VERSION:
    @echo "Setting version to {{VERSION}}"
    @cargo set-version --workspace {{VERSION}} || just set-version-manual {{VERSION}}

# Bump version (tries cargo-edit first, falls back to manual)
bump TYPE="patch":
    @echo "Bumping {{TYPE}} version"
    @cargo set-version --workspace --bump {{TYPE}} 2>/dev/null || just bump-manual {{TYPE}}

# Publish a single package
publish-package PACKAGE DRY="false":
    @echo "📦 Publishing {{PACKAGE}}..."
    @if [ "{{DRY}}" = "true" ]; then \
        cargo publish --package {{PACKAGE}} --dry-run 2>&1 || true; \
    else \
        cargo publish --package {{PACKAGE}} --allow-dirty; \
    fi

# Wait for crates.io to index
wait-for-index:
    @echo "⏳ Waiting 15 seconds for crates.io to index..."
    @sleep 15

# Modern release using cyrup_release (recommended)
release TYPE="patch":
    @echo "🚀 Starting release with cyrup_release..."
    cargo run --package cyrup_release -- release {{TYPE}} --verbose

# Dry run release using cyrup_release
release-dry TYPE="patch":
    @echo "🎭 Dry run release with cyrup_release..."
    cargo run --package cyrup_release -- release {{TYPE}} --dry-run --verbose

# Rollback a failed release
rollback:
    @echo "🔄 Rolling back release..."
    cargo run --package cyrup_release -- rollback --verbose

# Resume an interrupted release
resume:
    @echo "▶️ Resuming release..."
    cargo run --package cyrup_release -- resume --verbose

# Show release status
status:
    @echo "📊 Release status..."
    cargo run --package cyrup_release -- status --detailed

# Validate workspace for release
validate:
    @echo "✅ Validating workspace..."
    cargo run --package cyrup_release -- validate --detailed

# Preview version bump
preview TYPE="patch":
    @echo "🔍 Previewing {{TYPE}} version bump..."
    cargo run --package cyrup_release -- preview {{TYPE}} --detailed

# Legacy release (old perl-based approach - kept for emergency fallback)
release-legacy TYPE="patch":
    # Bump version
    @echo "⚠️ Using legacy release method..."
    @echo "Bumping {{TYPE}} version..."
    just bump {{TYPE}}
    # Get new version
    @echo "New version: $(just get-version)"
    # Update lock file
    cargo update --workspace
    # Commit all changes (including any uncommitted work)
    git add -A
    git diff --cached --quiet || git commit -m "release: v$(just get-version)"
    # Tag
    git tag -a "v$(just get-version)" -m "Release v$(just get-version)"
    @echo "🚀 Starting release of v$(just get-version)"
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
    git push origin "v$(just get-version)"
    @echo "✅ Release v$(just get-version) complete!"

# Legacy dry run release
release-dry-legacy TYPE="patch":
    # Check if ready
    just release-checklist
    @echo "🎭 DRY RUN - No actual publishing (legacy method)"
    # Show what would happen
    @echo "Would bump {{TYPE}} version"
    @echo "Current version: $(just get-version)"
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