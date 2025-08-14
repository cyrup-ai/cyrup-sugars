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

# Bump version (major, minor, or patch)
bump-version TYPE:
    #!/usr/bin/env bash
    set -euo pipefail
    current=$(just get-version)
    IFS='.' read -r major minor patch <<< "$current"
    
    case "{{TYPE}}" in
        major)
            new_version="$((major + 1)).0.0"
            ;;
        minor)
            new_version="${major}.$((minor + 1)).0"
            ;;
        patch)
            new_version="${major}.${minor}.$((patch + 1))"
            ;;
        *)
            echo "Invalid version type. Use: major, minor, or patch"
            exit 1
            ;;
    esac
    
    echo "Bumping version from $current to $new_version"
    
    # Update the workspace version
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
    else
        # Linux
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
    fi
    
    # Update lock file
    cargo update --workspace
    
    echo "$new_version"

# Release all packages in dependency order
release TYPE="patch" DRY_RUN="false":
    #!/usr/bin/env bash
    set -euo pipefail
    
    # Ensure working directory is clean
    if [[ -n $(git status --porcelain) ]]; then
        echo "Error: Working directory has uncommitted changes"
        exit 1
    fi
    
    # Bump version
    new_version=$(just bump-version {{TYPE}})
    
    # Run tests
    echo "Running tests..."
    cargo test --all-features
    cargo nextest run
    
    # Commit version bump
    git add -A
    git commit -m "chore: release v$new_version"
    
    # Create git tag
    git tag -a "v$new_version" -m "Release v$new_version"
    
    echo "🚀 Starting release of v$new_version"
    
    # Define release order based on dependencies
    declare -a tier0=("sugars_macros" "sugars_collections" "sugars_gix")
    declare -a tier1=("sugars_async_task")
    declare -a tier2=("sugars_async_stream" "sugars_builders" "sugars_llm")
    declare -a tier3=("cyrup_sugars")
    
    # Function to publish a package
    publish_package() {
        local package=$1
        local dry_run=$2
        echo "📦 Publishing $package..."
        
        if [[ "$dry_run" == "true" ]]; then
            echo "  [DRY RUN] Would publish: $package"
            cargo publish --package "$package" --dry-run
        else
            cargo publish --package "$package"
        fi
    }
    
    # Publish Tier 0 (no dependencies)
    echo "═══ Tier 0: Base packages (no dependencies) ═══"
    for package in "${tier0[@]}"; do
        publish_package "$package" "{{DRY_RUN}}"
        if [[ "{{DRY_RUN}}" != "true" ]]; then
            echo "⏳ Waiting 15 seconds for crates.io to index..."
            sleep 15
        fi
    done
    
    # Publish Tier 1
    echo "═══ Tier 1: First level dependencies ═══"
    for package in "${tier1[@]}"; do
        publish_package "$package" "{{DRY_RUN}}"
        if [[ "{{DRY_RUN}}" != "true" ]]; then
            echo "⏳ Waiting 15 seconds for crates.io to index..."
            sleep 15
        fi
    done
    
    # Publish Tier 2
    echo "═══ Tier 2: Second level dependencies ═══"
    for package in "${tier2[@]}"; do
        publish_package "$package" "{{DRY_RUN}}"
        if [[ "{{DRY_RUN}}" != "true" ]]; then
            echo "⏳ Waiting 15 seconds for crates.io to index..."
            sleep 15
        fi
    done
    
    # Publish Tier 3 (main package)
    echo "═══ Tier 3: Main package ═══"
    for package in "${tier3[@]}"; do
        publish_package "$package" "{{DRY_RUN}}"
    done
    
    # Push to git
    if [[ "{{DRY_RUN}}" != "true" ]]; then
        echo "📤 Pushing to git..."
        git push origin main
        git push origin "v$new_version"
        echo "✅ Release v$new_version complete!"
    else
        echo "✅ Dry run complete. No packages were actually published."
        echo "   To perform actual release, run: just release {{TYPE}}"
    fi

# Dry run release (no actual publishing)
release-dry TYPE="patch":
    just release {{TYPE}} true

# Check which packages would be published
check-publish:
    @echo "Checking publishable packages..."
    @for pkg in sugars_macros sugars_collections sugars_gix sugars_async_task sugars_async_stream sugars_builders sugars_llm cyrup_sugars; do \
        echo "Checking $pkg..."; \
        cargo publish --package "$pkg" --dry-run 2>&1 | grep -E "(Uploading|error)" || true; \
    done