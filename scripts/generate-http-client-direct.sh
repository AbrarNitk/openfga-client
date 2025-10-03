#!/bin/bash

# Generate Rust HTTP client from OpenAPI JSON using direct JAR download
# No Maven required - downloads JAR directly from Maven Central

set -e

# Configuration
OPENAPI_GENERATOR_VERSION="7.2.0"
JAR_NAME="openapi-generator-cli-${OPENAPI_GENERATOR_VERSION}.jar"
JAR_URL="https://repo1.maven.org/maven2/org/openapitools/openapi-generator-cli/${OPENAPI_GENERATOR_VERSION}/${JAR_NAME}"
CACHE_DIR=".bins/openapitools"
OUTPUT_DIR="openfga-http-client"
SPEC_FILE="client-builder/openapi/openfga-openapi.json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Generating Rust HTTP Client from OpenAPI JSON${NC}"
echo "=================================================="

# Create cache directory
mkdir -p "$CACHE_DIR"

# Download JAR if not exists
if [ ! -f "$CACHE_DIR/$JAR_NAME" ]; then
    echo -e "${YELLOW}📥 Downloading openapi-generator-cli JAR...${NC}"
    curl -L -o "$CACHE_DIR/$JAR_NAME" "$JAR_URL"
    echo -e "${GREEN}✅ Downloaded $JAR_NAME${NC}"
else
    echo -e "${GREEN}✅ Using cached $JAR_NAME${NC}"
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

echo -e "${YELLOW}🔧 Generating Rust HTTP client...${NC}"

# Generate the client
java -jar "$CACHE_DIR/$JAR_NAME" generate \
    -i "$SPEC_FILE" \
    -g rust \
    -o "$OUTPUT_DIR" \
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
