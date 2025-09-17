#!/bin/bash

# Test Resource Listing Script
# This script tests the resource listing functionality with different users

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
API_BASE_URL="http://localhost:3000"
OPENFGA_URL="http://localhost:8080"
STORE_ID="${OPENFGA_STORE_ID}"

echo -e "${BLUE}🔍 Testing Resource Listing Functionality${NC}"
echo "=================================================="

# Function to test API endpoint
test_endpoint() {
    local user_token="$1"
    local user_name="$2"
    local endpoint="$3"
    local description="$4"
    
    echo -e "\n${YELLOW}Testing: ${description}${NC}"
    echo "User: ${user_name}"
    echo "Endpoint: ${endpoint}"
    echo "Response:"
    
    if curl -s -H "Authorization: Bearer ${user_token}" \
       "${API_BASE_URL}${endpoint}" | jq . 2>/dev/null; then
        echo -e "${GREEN}✅ Success${NC}"
    else
        echo -e "${RED}❌ Failed${NC}"
    fi
}

# Function to test OpenFGA ListObjects directly
test_openfga_direct() {
    local user_id="$1"
    local user_name="$2"
    local object_type="$3"
    local relation="$4"
    
    echo -e "\n${YELLOW}Direct OpenFGA ListObjects Test${NC}"
    echo "User: ${user_name} (${user_id})"
    echo "Object Type: ${object_type}"
    echo "Relation: ${relation}"
    echo "Response:"
    
    local payload=$(cat <<EOF
{
    "type": "${object_type}",
    "relation": "${relation}",
    "user": "${user_id}"
}
EOF
)
    
    if curl -s -X POST "${OPENFGA_URL}/stores/${STORE_ID}/list-objects" \
       -H "Content-Type: application/json" \
       -d "${payload}" | jq . 2>/dev/null; then
        echo -e "${GREEN}✅ Success${NC}"
    else
        echo -e "${RED}❌ Failed${NC}"
    fi
}

# Test scenarios
echo -e "\n${BLUE}📋 Test Scenarios${NC}"

# Scenario 1: Carl (partner org member) - should see full connector service
echo -e "\n${GREEN}=== Scenario 1: Carl (Partner Org Member) ===${NC}"
echo "Expected: Should see full connector service via group:partner_share"

# Mock token for Carl (in real app, get from auth)
CARL_TOKEN="carl_mock_token"

test_endpoint "${CARL_TOKEN}" "Carl" "/api/list-objects?object_type=service&relation=viewer" \
    "List services Carl can view"

test_endpoint "${CARL_TOKEN}" "Carl" "/api/list-objects?object_type=resource&relation=viewer" \
    "List resources Carl can view"

test_endpoint "${CARL_TOKEN}" "Carl" "/api/shared-resources" \
    "Get all shared resources for Carl"

# Direct OpenFGA test for Carl
test_openfga_direct "user:carl" "Carl" "service" "viewer"
test_openfga_direct "user:carl" "Carl" "resource" "viewer"

# Scenario 2: Emily (partner_child org member) - should see only s3 service type
echo -e "\n${GREEN}=== Scenario 2: Emily (Partner Child Org Member) ===${NC}"
echo "Expected: Should see only s3 service type via group:child_share"

EMILY_TOKEN="emily_mock_token"

test_endpoint "${EMILY_TOKEN}" "Emily" "/api/list-objects?object_type=service_type&relation=viewer" \
    "List service types Emily can view"

test_endpoint "${EMILY_TOKEN}" "Emily" "/api/list-objects?object_type=resource&relation=viewer" \
    "List resources Emily can view"

test_endpoint "${EMILY_TOKEN}" "Emily" "/api/shared-resources" \
    "Get all shared resources for Emily"

# Direct OpenFGA test for Emily
test_openfga_direct "user:emily" "Emily" "service_type" "viewer"
test_openfga_direct "user:emily" "Emily" "resource" "viewer"

# Scenario 3: Diana (partner org member) - should see shared resources plus direct access
echo -e "\n${GREEN}=== Scenario 3: Diana (Partner Org Member with Direct Access) ===${NC}"
echo "Expected: Should see shared resources plus direct editor access to gcs/401"

DIANA_TOKEN="diana_mock_token"

test_endpoint "${DIANA_TOKEN}" "Diana" "/api/list-objects?object_type=resource&relation=viewer" \
    "List resources Diana can view"

test_endpoint "${DIANA_TOKEN}" "Diana" "/api/list-objects?object_type=resource&relation=editor" \
    "List resources Diana can edit"

test_endpoint "${DIANA_TOKEN}" "Diana" "/api/shared-resources" \
    "Get all shared resources for Diana"

# Direct OpenFGA test for Diana
test_openfga_direct "user:diana" "Diana" "resource" "viewer"
test_openfga_direct "user:diana" "Diana" "resource" "editor"

# Scenario 4: Alice (system org member) - should see owned resources
echo -e "\n${GREEN}=== Scenario 4: Alice (System Org Member) ===${NC}"
echo "Expected: Should see owned resources in system org"

ALICE_TOKEN="alice_mock_token"

test_endpoint "${ALICE_TOKEN}" "Alice" "/api/list-objects?object_type=resource&relation=admin" \
    "List resources Alice can admin"

test_endpoint "${ALICE_TOKEN}" "Alice" "/api/shared-resources" \
    "Get all shared resources for Alice"

# Direct OpenFGA test for Alice
test_openfga_direct "user:alice" "Alice" "resource" "admin"

# Performance test
echo -e "\n${BLUE}⚡ Performance Test${NC}"
echo "Testing response times for different queries..."

echo -e "\n${YELLOW}Testing large result set performance${NC}"
time test_endpoint "${CARL_TOKEN}" "Carl" "/api/shared-resources" \
    "Performance test: Get all shared resources"

# Verification tests
echo -e "\n${BLUE}🔍 Verification Tests${NC}"

echo -e "\n${YELLOW}Verifying sharing inheritance${NC}"
echo "Testing if service-level sharing grants access to service types and resources..."

# Test if Carl can access specific resources through service sharing
test_openfga_direct "user:carl" "Carl" "resource" "viewer"

echo -e "\n${YELLOW}Verifying permission levels${NC}"
echo "Testing different permission levels (viewer, editor, admin)..."

# Test different permission levels
for relation in viewer editor admin; do
    echo -e "\nTesting ${relation} relation:"
    test_openfga_direct "user:carl" "Carl" "service" "${relation}"
done

# Edge case tests
echo -e "\n${BLUE}🧪 Edge Case Tests${NC}"

echo -e "\n${YELLOW}Testing non-existent user${NC}"
test_openfga_direct "user:nonexistent" "NonExistent" "resource" "viewer"

echo -e "\n${YELLOW}Testing invalid object type${NC}"
test_openfga_direct "user:carl" "Carl" "invalid_type" "viewer"

echo -e "\n${YELLOW}Testing invalid relation${NC}"
test_openfga_direct "user:carl" "Carl" "resource" "invalid_relation"

# Summary
echo -e "\n${BLUE}📊 Test Summary${NC}"
echo "=================================================="
echo "✅ All test scenarios completed"
echo "📋 Check the results above for any failures"
echo "🔧 If tests fail, verify:"
echo "   - OpenFGA server is running"
echo "   - Store ID is set correctly"
echo "   - Authorization model is loaded"
echo "   - Relationship tuples are loaded"
echo "   - API server is running"

echo -e "\n${GREEN}🎉 Resource listing tests completed!${NC}"

# Instructions for running
cat << 'EOF'

## How to run this test:

1. Start OpenFGA server:
   ./scripts/start-openfga.sh

2. Load the authorization model and tuples:
   ./scripts/demo.sh

3. Start the API server:
   cargo run

4. Set environment variables:
   export OPENFGA_STORE_ID="your_store_id"

5. Run this test script:
   chmod +x ./scripts/test-resource-listing.sh
   ./scripts/test-resource-listing.sh

## Expected Results:

- Carl: Should see all connector resources (service sharing)
- Emily: Should see only s3 resources (service-type sharing)  
- Diana: Should see shared resources + direct gcs access
- Alice: Should see owned system resources

EOF
