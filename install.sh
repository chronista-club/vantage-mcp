#!/bin/bash
set -e

# Ichimi Server Installation Script
# Usage: curl -fsSL https://raw.githubusercontent.com/chronista-club/ichimi-server/main/install.sh | bash

REPO="chronista-club/ichimi-server"
INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="ichimi"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Print colored messages
info() { echo -e "${GREEN}[INFO]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"
    
    case "$OS" in
        Linux*)     PLATFORM="linux";;
        Darwin*)    PLATFORM="macos";;
        *)          error "Unsupported OS: $OS";;
    esac
    
    case "$ARCH" in
        x86_64)     ARCH="x86_64";;
        aarch64|arm64) ARCH="aarch64";;
        *)          error "Unsupported architecture: $ARCH";;
    esac
    
    echo "${PLATFORM}-${ARCH}"
}

# Get latest release URL from GitHub
get_latest_release_url() {
    local platform="$1"
    local api_url="https://api.github.com/repos/${REPO}/releases/latest"
    
    # Try to get the latest release
    local release_info=$(curl -sL "$api_url" 2>/dev/null)
    
    if [ -z "$release_info" ] || echo "$release_info" | grep -q "Not Found"; then
        warn "No releases found. Installing from source..."
        return 1
    fi
    
    # Extract download URL for the platform
    local download_url=$(echo "$release_info" | grep -o "\"browser_download_url\": \"[^\"]*${platform}[^\"]*\"" | cut -d'"' -f4 | head -1)
    
    if [ -z "$download_url" ]; then
        warn "No binary found for platform: ${platform}"
        return 1
    fi
    
    echo "$download_url"
}

# Install from binary release
install_from_release() {
    local platform=$(detect_platform)
    info "Detected platform: $platform"
    
    local download_url=$(get_latest_release_url "$platform")
    
    if [ $? -ne 0 ] || [ -z "$download_url" ]; then
        return 1
    fi
    
    info "Downloading from: $download_url"
    
    # Create temp directory
    local temp_dir=$(mktemp -d)
    trap "rm -rf $temp_dir" EXIT
    
    # Download and extract
    cd "$temp_dir"
    if ! curl -sL "$download_url" -o "${BINARY_NAME}.tar.gz"; then
        error "Failed to download from $download_url"
    fi
    
    if ! tar -xzf "${BINARY_NAME}.tar.gz"; then
        error "Failed to extract archive"
    fi
    
    # Check if binary exists
    if [ ! -f "$BINARY_NAME" ]; then
        error "Binary not found after extraction"
    fi
    
    # Install binary
    mkdir -p "$INSTALL_DIR"
    mv "$BINARY_NAME" "$INSTALL_DIR/"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    
    info "Successfully installed to ${INSTALL_DIR}/${BINARY_NAME}"
    return 0
}

# Install from source using cargo
install_from_source() {
    info "Installing from source..."
    
    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        error "Rust is not installed. Please install Rust first: https://rustup.rs/"
    fi
    
    info "Installing ichimi-server using cargo..."
    cargo install --git "https://github.com/${REPO}" --bin ichimi
    
    info "Successfully installed using cargo"
    return 0
}

# Install using Homebrew (macOS)
install_with_homebrew() {
    # Temporarily disabled until Homebrew tap is available
    return 1
    
    # if command -v brew &> /dev/null; then
    #     info "Installing with Homebrew..."
    #     brew tap chronista-club/tap 2>/dev/null || true
    #     brew install ichimi-server
    #     return $?
    # fi
    # return 1
}

# Main installation flow
main() {
    echo "Installing Ichimi Server..."
    echo ""
    
    # Try Homebrew first on macOS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        if install_with_homebrew; then
            echo ""
            info "Installation complete!"
            info "Run 'ichimi --help' to get started"
            exit 0
        fi
    fi
    
    # Try binary release
    if install_from_release; then
        # Add to PATH if needed
        if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
            warn "${INSTALL_DIR} is not in your PATH"
            info "Add the following to your shell configuration:"
            echo "  export PATH=\"\$PATH:${INSTALL_DIR}\""
        fi
        echo ""
        info "Installation complete!"
        info "Run 'ichimi --help' to get started"
        exit 0
    fi
    
    # Fall back to source installation
    if install_from_source; then
        echo ""
        info "Installation complete!"
        info "Run 'ichimi --help' to get started"
        exit 0
    fi
    
    error "Installation failed. Please try manual installation."
}

# Run main function
main "$@"