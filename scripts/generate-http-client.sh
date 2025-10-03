#!/bin/bash

# Main script to generate Rust HTTP client - tries different methods
# Automatically selects the best available method based on installed tools

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 OpenFGA Rust HTTP Client Generator${NC}"
echo "====================================="

# Check what tools are available
HAS_DOCKER=$(command -v docker &> /dev/null && echo "yes" || echo "no")
HAS_NPX=$(command -v npx &> /dev/null && echo "yes" || echo "no")
HAS_JAVA=$(command -v java &> /dev/null && echo "yes" || echo "no")

echo -e "${BLUE}🔍 Checking available tools:${NC}"
echo "  Docker: $HAS_DOCKER"
echo "  Node.js/npx: $HAS_NPX"
echo "  Java: $HAS_JAVA"
echo ""

# Select method based on availability
if [ "$HAS_DOCKER" = "yes" ]; then
    echo -e "${GREEN}✅ Using Docker method (recommended)${NC}"
    exec ./scripts/generate-http-client-docker.sh
elif [ "$HAS_NPX" = "yes" ]; then
    echo -e "${GREEN}✅ Using npm/npx method${NC}"
    exec ./scripts/generate-http-client-npm.sh
elif [ "$HAS_JAVA" = "yes" ]; then
    echo -e "${GREEN}✅ Using direct JAR download method${NC}"
    exec ./scripts/generate-http-client-direct.sh
else
    echo -e "${RED}❌ No suitable tools found!${NC}"
    echo ""
    echo -e "${YELLOW}Please install one of the following:${NC}"
    echo "1. Docker (recommended): https://docs.docker.com/get-docker/"
    echo "2. Node.js/npm: https://nodejs.org/"
    echo "3. Java 11+: https://openjdk.org/"
    echo ""
    echo -e "${YELLOW}Then run this script again.${NC}"
    exit 1
fi
