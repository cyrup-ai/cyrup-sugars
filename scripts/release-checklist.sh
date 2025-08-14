#!/usr/bin/env bash
# Pre-release checklist

echo "📋 Release Checklist"
echo "==================="

# Check for uncommitted changes
if [[ -n $(git status --porcelain) ]]; then
    echo "❌ Uncommitted changes detected"
    git status --short
    exit 1
else
    echo "✅ Working directory clean"
fi

# Check all tests pass
echo "🧪 Running tests..."
if cargo test --all-features --quiet; then
    echo "✅ All tests pass"
else
    echo "❌ Tests failed"
    exit 1
fi

# Check nextest if available
if command -v cargo-nextest &> /dev/null; then
    echo "🧪 Running nextest..."
    if cargo nextest run --quiet; then
        echo "✅ Nextest passes"
    else
        echo "❌ Nextest failed"
        exit 1
    fi
fi

# Check documentation builds
echo "📚 Building documentation..."
if cargo doc --all-features --no-deps --quiet; then
    echo "✅ Documentation builds"
else
    echo "❌ Documentation build failed"
    exit 1
fi

# Check for TODO items in code
echo "🔍 Checking for TODOs..."
todo_count=$(grep -r "TODO\|FIXME\|XXX" --include="*.rs" packages/ 2>/dev/null | wc -l | tr -d ' ')
if [ "$todo_count" -gt 0 ]; then
    echo "⚠️  Found $todo_count TODO/FIXME items"
    grep -r "TODO\|FIXME\|XXX" --include="*.rs" packages/ | head -5
    echo "   ..."
else
    echo "✅ No TODO/FIXME items"
fi

# Check clippy
echo "📎 Running clippy..."
if cargo clippy --all-targets --all-features -- -D warnings 2>&1 | grep -q "warning\|error"; then
    echo "❌ Clippy found issues"
    cargo clippy --all-targets --all-features -- -D warnings
    exit 1
else
    echo "✅ Clippy clean"
fi

# Check formatting
echo "🎨 Checking formatting..."
if cargo fmt --all -- --check; then
    echo "✅ Code is formatted"
else
    echo "❌ Code needs formatting"
    echo "   Run: cargo fmt --all"
    exit 1
fi

# Check examples compile
echo "🔧 Checking examples..."
if cargo build --examples --quiet; then
    echo "✅ All examples compile"
else
    echo "❌ Example compilation failed"
    exit 1
fi

# Display current version
echo ""
echo "📦 Current version: $(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)"
echo ""
echo "✅ Ready for release!"
echo ""
echo "Next steps:"
echo "  1. Review changes: git diff HEAD~1"
echo "  2. Dry run: just release-dry [major|minor|patch]"
echo "  3. Release: just release [major|minor|patch]"