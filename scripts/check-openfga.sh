#!/bin/bash

# Script to check OpenFGA server status and help with setup

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}OpenFGA Server Status Check${NC}"
echo "============================"
echo ""

# Check common OpenFGA ports
PORTS=(8080 8081 8082)
OPENFGA_FOUND=false

for port in "${PORTS[@]}"; do
    echo -n "Checking port $port... "
    if curl -s -f "http://localhost:$port/healthz" >/dev/null 2>&1; then
        echo -e "${GREEN}OpenFGA server found!${NC}"
        echo "OpenFGA is running on http://localhost:$port"
        OPENFGA_FOUND=true
        
        # Update environment variable suggestion
        echo ""
        echo -e "${YELLOW}Suggested environment configuration:${NC}"
        echo "export OPENFGA_CLIENT_URL=http://localhost:$port"
        echo ""
        break
    elif curl -s "http://localhost:$port" >/dev/null 2>&1; then
        echo -e "${YELLOW}Service found but not OpenFGA${NC}"
    else
        echo -e "${RED}No service${NC}"
    fi
done

echo ""

if [ "$OPENFGA_FOUND" = false ]; then
    echo -e "${RED}OpenFGA server not found on common ports.${NC}"
    echo ""
    echo -e "${YELLOW}To start OpenFGA server:${NC}"
    echo ""
    echo "1. Using Docker:"
    echo "   docker run --rm -it -p 8080:8080 -p 8081:8081 -p 3000:3000 openfga/openfga run"
    echo ""
    echo "2. Using Docker Compose (if you have docker-compose.yml):"
    echo "   docker-compose up openfga"
    echo ""
    echo "3. Download and run binary:"
    echo "   # Download from https://github.com/openfga/openfga/releases"
    echo "   ./openfga run --playground-enabled"
    echo ""
    echo -e "${YELLOW}Common OpenFGA ports:${NC}"
    echo "   - 8080: HTTP API"
    echo "   - 8081: gRPC API"  
    echo "   - 3000: Playground (web UI)"
    echo ""
else
    echo -e "${GREEN}OpenFGA server is running!${NC}"
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo "1. Set environment variables (source etc/envs.sh or create .env file)"
    echo "2. Create a store and authorization model in OpenFGA"
    echo "3. Update OPENFGA_STORE_ID and OPENFGA_AUTH_MODEL_ID in your environment"
    echo ""
    echo "You can use the OpenFGA playground at http://localhost:3000 (if enabled)"
fi

echo ""
echo -e "${BLUE}Current environment variables:${NC}"
echo "OPENFGA_CLIENT_URL: ${OPENFGA_CLIENT_URL:-not set}"
echo "OPENFGA_STORE_ID: ${OPENFGA_STORE_ID:-not set}"
echo "OPENFGA_AUTH_MODEL_ID: ${OPENFGA_AUTH_MODEL_ID:-not set}"
