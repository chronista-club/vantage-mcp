#!/bin/bash

# Release helper script for Ichimi Server
# Usage: ./scripts/release.sh [patch|minor|major|beta]

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo -e "${YELLOW}Current version: ${CURRENT_VERSION}${NC}"

# Parse version components
if [[ $CURRENT_VERSION =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)(-beta([0-9]+))?$ ]]; then
    MAJOR="${BASH_REMATCH[1]}"
    MINOR="${BASH_REMATCH[2]}"
    PATCH="${BASH_REMATCH[3]}"
    BETA_NUM="${BASH_REMATCH[5]}"
else
    echo -e "${RED}Error: Unable to parse current version${NC}"
    exit 1
fi

# Determine new version based on argument
case "$1" in
    major)
        NEW_VERSION="$((MAJOR + 1)).0.0"
        ;;
    minor)
        NEW_VERSION="${MAJOR}.$((MINOR + 1)).0"
        ;;
    patch)
        NEW_VERSION="${MAJOR}.${MINOR}.$((PATCH + 1))"
        ;;
    beta)
        if [ -z "$BETA_NUM" ]; then
            NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}-beta1"
        else
            NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}-beta$((BETA_NUM + 1))"
        fi
        ;;
    *)
        echo "Usage: $0 [patch|minor|major|beta]"
        echo ""
        echo "Examples:"
        echo "  $0 patch  # 0.1.0 -> 0.1.1"
        echo "  $0 minor  # 0.1.0 -> 0.2.0"
        echo "  $0 major  # 0.1.0 -> 1.0.0"
        echo "  $0 beta   # 0.1.0 -> 0.1.0-beta1 or 0.1.0-beta1 -> 0.1.0-beta2"
        exit 1
        ;;
esac

echo -e "${GREEN}New version will be: ${NEW_VERSION}${NC}"
echo ""

# Confirm with user
read -p "Continue with release? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Release cancelled"
    exit 1
fi

# Update Cargo.toml
echo "Updating Cargo.toml..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/^version = \".*\"/version = \"${NEW_VERSION}\"/" Cargo.toml
else
    # Linux
    sed -i "s/^version = \".*\"/version = \"${NEW_VERSION}\"/" Cargo.toml
fi

# Update Cargo.lock
echo "Updating Cargo.lock..."
cargo update --workspace

# Run tests
echo "Running tests..."
if ! cargo test; then
    echo -e "${RED}Tests failed! Aborting release.${NC}"
    git checkout Cargo.toml Cargo.lock
    exit 1
fi

# Commit changes
echo "Committing version update..."
git add Cargo.toml Cargo.lock
git commit -m "chore: バージョンを${NEW_VERSION}に更新

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"

# Create and push tag
echo "Creating tag v${NEW_VERSION}..."
git tag -a "v${NEW_VERSION}" -m "Release v${NEW_VERSION}"

# Push changes
echo "Pushing to remote..."
git push origin main
git push origin "v${NEW_VERSION}"

echo ""
echo -e "${GREEN}✅ Release v${NEW_VERSION} initiated successfully!${NC}"
echo ""
echo "GitHub Actions will now:"
echo "  1. Run tests on multiple platforms"
echo "  2. Build release binaries"
echo "  3. Create GitHub release"
echo ""
echo "Monitor progress at: https://github.com/chronista-club/ichimi-server/actions"