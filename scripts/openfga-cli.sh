#!/bin/bash

# OpenFGA CLI Installation Script
# Automatically detects the best installation method and installs the OpenFGA CLI
# Based on the methods documented in the OpenFGA CLI repository

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
DEFAULT_VERSION="v0.7.4"
INSTALL_DIR="/usr/local/bin"
TEMP_DIR="/tmp/openfga-cli-install"

echo -e "${GREEN}🚀 OpenFGA CLI Installation Script${NC}"
echo "=================================="
echo ""

# Function to detect OS and architecture
detect_system() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    
    # Map architecture names to OpenFGA release names
    case $ARCH in
        x86_64)
            ARCH="amd64"
            ;;
        aarch64|arm64)
            ARCH="arm64"
            ;;
        armv7l)
            ARCH="armv7"
            ;;
        i386|i686)
            ARCH="386"
            ;;
        *)
            echo -e "${RED}❌ Unsupported architecture: $ARCH${NC}"
            exit 1
            ;;
    esac
    
    echo -e "${BLUE}🔍 Detected system: ${OS} ${ARCH}${NC}"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to get latest version from GitHub API
get_latest_version() {
    if command_exists curl; then
        echo -e "${BLUE}🔍 Fetching latest version from GitHub...${NC}"
        LATEST_VERSION=$(curl -s https://api.github.com/repos/openfga/cli/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
        if [ -n "$LATEST_VERSION" ] && [ "$LATEST_VERSION" != "null" ]; then
            echo -e "${GREEN}✅ Latest version found: $LATEST_VERSION${NC}"
            echo "$LATEST_VERSION"
        else
            echo -e "${YELLOW}⚠️  Could not fetch latest version, using default: $DEFAULT_VERSION${NC}"
            echo "$DEFAULT_VERSION"
        fi
    else
        echo -e "${YELLOW}⚠️  curl not available, using default version: $DEFAULT_VERSION${NC}"
        echo "$DEFAULT_VERSION"
    fi
}

# Function to install via Homebrew (macOS/Linux)
install_via_brew() {
    echo -e "${GREEN}✅ Installing via Homebrew${NC}"
    if ! command_exists brew; then
        echo -e "${RED}❌ Homebrew not found${NC}"
        return 1
    fi
    
    brew tap openfga/tap
    brew install openfga/tap/fga
    return 0
}

# Function to install via Go
install_via_go() {
    echo -e "${GREEN}✅ Installing via Go${NC}"
    if ! command_exists go; then
        echo -e "${RED}❌ Go not found${NC}"
        return 1
    fi
    
    go install github.com/openfga/cli/cmd/fga@latest
    return 0
}

# Function to install via Docker (creates wrapper script)
install_via_docker() {
    echo -e "${GREEN}✅ Creating Docker wrapper${NC}"
    if ! command_exists docker; then
        echo -e "${RED}❌ Docker not found${NC}"
        return 1
    fi
    
    # Create wrapper script
    sudo tee "$INSTALL_DIR/fga" > /dev/null <<'EOF'
#!/bin/bash
docker run --rm -it -v "$(pwd):/workspace" -w /workspace openfga/cli:latest "$@"
EOF
    
    sudo chmod +x "$INSTALL_DIR/fga"
    echo -e "${CYAN}ℹ️  Docker wrapper created at $INSTALL_DIR/fga${NC}"
    return 0
}

# Function to download and install binary
install_via_binary() {
    local version="$1"
    echo -e "${GREEN}✅ Installing via binary download (version: $version)${NC}"
    
    if ! command_exists curl && ! command_exists wget; then
        echo -e "${RED}❌ Neither curl nor wget found${NC}"
        return 1
    fi
    
    # Create temporary directory
    mkdir -p "$TEMP_DIR"
    cd "$TEMP_DIR"
    
    # Construct download URL
    local filename="fga_${version#v}_${OS}_${ARCH}.tar.gz"
    local url="https://github.com/openfga/cli/releases/download/${version}/${filename}"
    
    echo -e "${BLUE}📥 Downloading from: $url${NC}"
    
    # Download
    if command_exists curl; then
        curl -L -o "$filename" "$url"
    else
        wget -O "$filename" "$url"
    fi
    
    if [ ! -f "$filename" ]; then
        echo -e "${RED}❌ Download failed${NC}"
        return 1
    fi
    
    # Extract
    echo -e "${BLUE}📦 Extracting archive...${NC}"
    tar -xzf "$filename"
    
    if [ ! -f "fga" ]; then
        echo -e "${RED}❌ Binary not found in archive${NC}"
        return 1
    fi
    
    # Install
    echo -e "${BLUE}📋 Installing to $INSTALL_DIR...${NC}"
    sudo mv fga "$INSTALL_DIR/"
    sudo chmod +x "$INSTALL_DIR/fga"
    
    # Cleanup
    cd /
    rm -rf "$TEMP_DIR"
    
    return 0
}

# Function to verify installation
verify_installation() {
    echo -e "${BLUE}🔍 Verifying installation...${NC}"
    
    if command_exists fga; then
        local version_output
        version_output=$(fga --version 2>/dev/null || fga version 2>/dev/null || echo "unknown")
        echo -e "${GREEN}✅ OpenFGA CLI installed successfully!${NC}"
        echo -e "${CYAN}Version: $version_output${NC}"
        echo ""
        echo -e "${YELLOW}💡 Quick start:${NC}"
        echo "  fga --help                    # Show help"
        echo "  fga store create --name test  # Create a store"
        echo "  fga store list               # List stores"
        return 0
    else
        echo -e "${RED}❌ Installation verification failed${NC}"
        return 1
    fi
}

# Main installation logic
main() {
    detect_system
    
    # Check what tools are available
    local has_brew=$(command_exists brew && echo "yes" || echo "no")
    local has_go=$(command_exists go && echo "yes" || echo "no")
    local has_docker=$(command_exists docker && echo "yes" || echo "no")
    local has_curl=$(command_exists curl && echo "yes" || echo "no")
    local has_wget=$(command_exists wget && echo "yes" || echo "no")
    
    echo ""
    echo -e "${BLUE}🔍 Checking available installation methods:${NC}"
    echo "  Homebrew: $has_brew"
    echo "  Go: $has_go"
    echo "  Docker: $has_docker"
    echo "  curl/wget: $([ "$has_curl" = "yes" ] || [ "$has_wget" = "yes" ] && echo "yes" || echo "no")"
    echo ""
    
    # Try installation methods in order of preference
    local version
    version=$(get_latest_version)
    
    # Method 1: Homebrew (if on macOS or Linux with Homebrew)
    if [ "$has_brew" = "yes" ]; then
        echo -e "${GREEN}🍺 Trying Homebrew installation...${NC}"
        if install_via_brew; then
            verify_installation
            return 0
        fi
        echo -e "${YELLOW}⚠️  Homebrew installation failed, trying next method...${NC}"
        echo ""
    fi
    
    # Method 2: Go installation
    if [ "$has_go" = "yes" ]; then
        echo -e "${GREEN}🐹 Trying Go installation...${NC}"
        if install_via_go; then
            verify_installation
            return 0
        fi
        echo -e "${YELLOW}⚠️  Go installation failed, trying next method...${NC}"
        echo ""
    fi
    
    # Method 3: Binary download
    if [ "$has_curl" = "yes" ] || [ "$has_wget" = "yes" ]; then
        echo -e "${GREEN}📥 Trying binary download installation...${NC}"
        if install_via_binary "$version"; then
            verify_installation
            return 0
        fi
        echo -e "${YELLOW}⚠️  Binary installation failed, trying next method...${NC}"
        echo ""
    fi
    
    # Method 4: Docker wrapper (last resort)
    if [ "$has_docker" = "yes" ]; then
        echo -e "${GREEN}🐳 Trying Docker wrapper installation...${NC}"
        if install_via_docker; then
            verify_installation
            return 0
        fi
        echo -e "${YELLOW}⚠️  Docker installation failed${NC}"
        echo ""
    fi
    
    # All methods failed
    echo -e "${RED}❌ All installation methods failed!${NC}"
    echo ""
    echo -e "${YELLOW}📝 Manual installation options:${NC}"
    echo "1. Install Homebrew: https://brew.sh/"
    echo "2. Install Go: https://golang.org/dl/"
    echo "3. Install Docker: https://docs.docker.com/get-docker/"
    echo "4. Download manually: https://github.com/openfga/cli/releases"
    echo ""
    echo -e "${YELLOW}💡 For package-specific installations:${NC}"
    echo "  # Debian/Ubuntu:"
    echo "  wget https://github.com/openfga/cli/releases/download/$version/fga_${version#v}_linux_amd64.deb"
    echo "  sudo apt install ./fga_${version#v}_linux_amd64.deb"
    echo ""
    echo "  # RHEL/Fedora:"
    echo "  wget https://github.com/openfga/cli/releases/download/$version/fga_${version#v}_linux_amd64.rpm"
    echo "  sudo dnf install ./fga_${version#v}_linux_amd64.rpm"
    
    exit 1
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "OpenFGA CLI Installation Script"
        echo ""
        echo "Usage: $0 [OPTIONS]"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --version, -v  Show version information"
        echo ""
        echo "This script automatically detects the best installation method"
        echo "and installs the OpenFGA CLI on your system."
        echo ""
        echo "Supported installation methods (in order of preference):"
        echo "  1. Homebrew (macOS/Linux)"
        echo "  2. Go (go install)"
        echo "  3. Binary download (curl/wget)"
        echo "  4. Docker wrapper"
        exit 0
        ;;
    --version|-v)
        echo "OpenFGA CLI Installation Script v1.0"
        exit 0
        ;;
    *)
        # Run main installation
        main
        ;;
esac