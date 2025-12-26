#!/usr/bin/env bash
#
# Installs git hooks for local development.
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_DIR="$REPO_ROOT/.git/hooks"

echo "📦 Installing git hooks..."

# Install pre-commit hook
if [ -f "$HOOKS_DIR/pre-commit" ]; then
    echo "   ⚠️  Backing up existing pre-commit hook to pre-commit.bak"
    mv "$HOOKS_DIR/pre-commit" "$HOOKS_DIR/pre-commit.bak"
fi

ln -sf "$SCRIPT_DIR/pre-commit" "$HOOKS_DIR/pre-commit"
echo "   ✓ Installed pre-commit hook"

echo ""
echo "✅ Git hooks installed successfully!"
echo ""
echo "The pre-commit hook will run the following checks before each commit:"
echo "  • cargo fmt -- --check"
echo "  • cargo clippy --all-targets --all-features -- -D warnings"
echo "  • cargo test --all-targets --all-features"
echo ""
echo "To skip hooks temporarily, use: git commit --no-verify"

