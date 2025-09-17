#!/bin/bash

# Demo script to showcase all API functionality
# Make sure the OpenFGA Demo server is running on localhost:3000

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
API_SCRIPT="$SCRIPT_DIR/api-test.sh"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}OpenFGA Demo API - Full Demo${NC}"
echo "=================================="
echo ""

# Test public endpoints
echo -e "${GREEN}1. Testing Public Endpoints${NC}"
echo "----------------------------"
echo ""

echo "Testing health endpoint..."
$API_SCRIPT alice health
echo ""

echo "Testing root endpoint..."
$API_SCRIPT alice root
echo ""

# Test resource operations with alice
echo -e "${GREEN}2. Testing Resource Operations (User: alice)${NC}"
echo "----------------------------------------------"
echo ""

echo "Creating a resource..."
$API_SCRIPT alice create my-service web org-1 demo-resource
echo ""

echo "Getting the resource..."
$API_SCRIPT alice get my-service web org-1 demo-resource
echo ""

echo "Updating the resource..."
$API_SCRIPT alice update my-service web org-1 demo-resource
echo ""

echo "Deleting the resource..."
$API_SCRIPT alice delete my-service web org-1 demo-resource
echo ""

# Test with different user
echo -e "${GREEN}3. Testing with Different User (User: bob)${NC}"
echo "--------------------------------------------"
echo ""

echo "Bob trying to create a resource..."
$API_SCRIPT bob create my-app api org-2 bob-resource
echo ""

echo "Bob trying to get his resource..."
$API_SCRIPT bob get my-app api org-2 bob-resource
echo ""

# Test with custom payloads
echo -e "${GREEN}4. Testing with Custom Payloads${NC}"
echo "--------------------------------"
echo ""

echo "Creating resource with custom payload..."
$API_SCRIPT charlie create my-service web org-1 payload-test $SCRIPT_DIR/payloads/create-resource.json
echo ""

echo "Updating resource with custom payload..."
$API_SCRIPT charlie update my-service web org-1 payload-test $SCRIPT_DIR/payloads/update-resource.json
echo ""

# Test authentication errors
echo -e "${GREEN}5. Testing Authentication Scenarios${NC}"
echo "------------------------------------"
echo ""

echo "Testing without user header (should fail)..."
curl -s -X GET http://localhost:3000/api/resource/my-service/web/org-1/test-resource | jq . 2>/dev/null || echo "Request failed as expected"
echo ""

echo "Testing with empty user header (should fail)..."
curl -s -H "X-User-Id: " -X GET http://localhost:3000/api/resource/my-service/web/org-1/test-resource | jq . 2>/dev/null || echo "Request failed as expected"
echo ""

echo -e "${YELLOW}Demo completed!${NC}"
echo ""
echo "You can now test individual endpoints using:"
echo "  $API_SCRIPT <user-id> <action> [parameters...]"
echo ""
echo "Examples:"
echo "  $API_SCRIPT alice create"
echo "  $API_SCRIPT bob get my-service web org-1 my-resource"
echo "  $API_SCRIPT charlie update my-app api org-2 my-api payloads/update-resource.json"
