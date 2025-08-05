#!/bin/bash

# Release script for typst-oxide
# Usage: ./scripts/release.sh <version>
# Example: ./scripts/release.sh 0.1.0

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if version is provided
if [ $# -eq 0 ]; then
    print_error "Please provide a version number"
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.0"
    exit 1
fi

VERSION=$1

# Validate version format (basic semver check)
if ! [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    print_error "Invalid version format. Please use semantic versioning (e.g., 1.0.0)"
    exit 1
fi

print_info "Starting release process for version $VERSION"

# Check if we're on main/master branch
CURRENT_BRANCH=$(git branch --show-current)
if [[ "$CURRENT_BRANCH" != "main" && "$CURRENT_BRANCH" != "master" ]]; then
    print_warning "You are not on main/master branch (current: $CURRENT_BRANCH)"
    read -p "Do you want to continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Release cancelled"
        exit 0
    fi
fi

# Check if working directory is clean
if [[ -n $(git status --porcelain) ]]; then
    print_error "Working directory is not clean. Please commit or stash your changes."
    git status --short
    exit 1
fi

# Update version in Cargo.toml
print_info "Updating version in Cargo.toml"
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# Run tests
print_info "Running tests"
if ! cargo test; then
    print_error "Tests failed. Please fix the issues before releasing."
    exit 1
fi

# Check formatting
print_info "Checking code formatting"
if ! cargo fmt --all -- --check; then
    print_error "Code is not properly formatted. Run 'cargo fmt' to fix."
    exit 1
fi

# Run clippy
print_info "Running clippy"
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    print_error "Clippy found issues. Please fix them before releasing."
    exit 1
fi

# Build release
print_info "Building release version"
if ! cargo build --release; then
    print_error "Release build failed"
    exit 1
fi

# Commit version change
print_info "Committing version change"
git add Cargo.toml Cargo.lock
git commit -m "Bump version to $VERSION"

# Create and push tag
print_info "Creating and pushing tag v$VERSION"
git tag "v$VERSION"
git push origin "$CURRENT_BRANCH"
git push origin "v$VERSION"

print_info "Release process completed successfully!"
print_info "GitHub Actions will now handle the publication to crates.io"
print_info "You can monitor the progress at: https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:\/]\([^.]*\).*/\1/')/actions"

# Optional: Open GitHub Actions page
read -p "Do you want to open GitHub Actions page? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    REPO_URL=$(git config --get remote.origin.url | sed 's/git@github.com:/https:\/\/github.com\//' | sed 's/\.git$//')
    open "$REPO_URL/actions" 2>/dev/null || xdg-open "$REPO_URL/actions" 2>/dev/null || echo "Please manually open: $REPO_URL/actions"
fi