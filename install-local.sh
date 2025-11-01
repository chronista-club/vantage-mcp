#!/bin/bash
# Local install script for vantage-mcp

set -e

# Color output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="vantagemcp"
BUILD_MODE="${1:-release}"

echo -e "${YELLOW}🚀 Installing vantage-mcp locally...${NC}"

# Create install directory if it doesn't exist
if [ ! -d "$INSTALL_DIR" ]; then
    echo -e "${YELLOW}Creating install directory: $INSTALL_DIR${NC}"
    mkdir -p "$INSTALL_DIR"
fi

# Build the project
if [ "$BUILD_MODE" = "release" ]; then
    echo -e "${YELLOW}Building in release mode (optimized)...${NC}"
    cargo build --release
    SOURCE_PATH="target/release/$BINARY_NAME"
elif [ "$BUILD_MODE" = "debug" ]; then
    echo -e "${YELLOW}Building in debug mode (fast)...${NC}"
    cargo build
    SOURCE_PATH="target/debug/$BINARY_NAME"
else
    echo -e "${RED}Invalid build mode: $BUILD_MODE${NC}"
    echo "Usage: $0 [release|debug]"
    exit 1
fi

# Check if build was successful
if [ ! -f "$SOURCE_PATH" ]; then
    echo -e "${RED}Build failed! Binary not found at: $SOURCE_PATH${NC}"
    exit 1
fi

# Copy binary to install directory
echo -e "${YELLOW}Installing to: $INSTALL_DIR/$BINARY_NAME${NC}"
cp "$SOURCE_PATH" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

# Get version
VERSION=$("$INSTALL_DIR/$BINARY_NAME" --version 2>/dev/null || echo "unknown")

# Check if install directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}⚠️  Warning: $INSTALL_DIR is not in your PATH${NC}"
    echo -e "Add the following to your shell configuration file (.bashrc, .zshrc, etc.):"
    echo -e "${GREEN}export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
fi

echo -e "${GREEN}✅ Successfully installed vantage-mcp!${NC}"
echo -e "Version: $VERSION"
echo -e "Location: $INSTALL_DIR/$BINARY_NAME"
echo -e ""
echo -e "You can now run: ${GREEN}vantage${NC}"

# Optional: Show current installation info
if command -v vantage &> /dev/null; then
    CURRENT_PATH=$(which vantage)
    if [ "$CURRENT_PATH" != "$INSTALL_DIR/$BINARY_NAME" ]; then
        echo -e "${YELLOW}Note: Another version exists at: $CURRENT_PATH${NC}"
    fi
fi
