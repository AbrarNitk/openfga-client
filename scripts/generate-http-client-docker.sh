#!/bin/bash

# Generate Rust HTTP client using Docker - no Java/Maven required
# Uses the official OpenAPI Generator Docker image

set -e

# Configuration
OUTPUT_DIR="openfga-http-client"
SPEC_FILE="client-builder/openapi/openfga-openapi.json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🐳 Generating Rust HTTP Client using Docker${NC}"
echo "=============================================="

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker is not installed or not in PATH${NC}"
    echo "Please install Docker to use this method"
    exit 1
fi

# Check if OpenAPI spec exists
if [ ! -f "$SPEC_FILE" ]; then
    echo -e "${RED}❌ OpenAPI spec not found: $SPEC_FILE${NC}"
    echo "Run 'cargo build' in client-builder/ to generate the OpenAPI spec first"
    exit 1
fi

# Clean output directory
if [ -d "$OUTPUT_DIR" ]; then
    echo -e "${YELLOW}🧹 Cleaning existing output directory...${NC}"
    rm -rf "$OUTPUT_DIR"
fi

echo -e "${YELLOW}🔧 Generating Rust HTTP client with Docker...${NC}"

# Generate the client using Docker
docker run --rm \
    -v "${PWD}:/local" \
    openapitools/openapi-generator-cli:v7.2.0 generate \
    -i "/local/$SPEC_FILE" \
    -g rust \
    -o "/local/$OUTPUT_DIR" \
    --additional-properties=packageName=openfga_http_client \
    --additional-properties=packageVersion=0.1.0 \
    --additional-properties=library=reqwest \
    --additional-properties=supportAsync=true \
    --additional-properties=useSingleRequestParameter=false \
    --package-name openfga_http_client

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Rust HTTP client generated successfully!${NC}"
    echo -e "${GREEN}📁 Output directory: $OUTPUT_DIR${NC}"
    echo ""
    echo -e "${YELLOW}📝 Next steps:${NC}"
    echo "1. Review the generated client in $OUTPUT_DIR/"
    echo "2. Add it to your workspace Cargo.toml if needed"
    echo "3. Use the client in your application"
else
    echo -e "${RED}❌ Failed to generate Rust HTTP client${NC}"
    exit 1
fi
