#!/bin/bash

# API Testing Script for OpenFGA Demo
# Usage: ./api-test.sh <user-id> <action> [service-name] [service-type] [org-id] [resource-name] [payload-file]

# Default configuration
BASE_URL="http://localhost:5001"
CONTENT_TYPE="application/json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
DEFAULT_SERVICE_NAME="connector"
DEFAULT_SERVICE_TYPE="web"
DEFAULT_ORG_ID="org-1"
DEFAULT_RESOURCE_NAME="my-resource"

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 <user-id> <action> [service-name] [service-type] [org-id] [resource-name] [payload-file]"
    echo ""
    echo "Actions:"
    echo "  health    - Check API health (no authentication required)"
    echo "  root      - Get root endpoint (no authentication required)"
    echo "  create    - Create a resource (POST)"
    echo "  get       - Get a resource (GET)"
    echo "  update    - Update a resource (PUT)"
    echo "  delete    - Delete a resource (DELETE)"
    echo ""
    echo "Parameters:"
    echo "  user-id       - User ID to send in X-User-Id header"
    echo "  service-name  - Service name (default: $DEFAULT_SERVICE_NAME)"
    echo "  service-type  - Service type (default: $DEFAULT_SERVICE_TYPE)"
    echo "  org-id        - Organization ID (default: $DEFAULT_ORG_ID)"
    echo "  resource-name - Resource name (default: $DEFAULT_RESOURCE_NAME)"
    echo "  payload-file  - JSON file for POST/PUT requests (optional)"
    echo ""
    echo "Examples:"
    echo "  $0 alice health"
    echo "  $0 alice create"
    echo "  $0 alice create my-app api org-2 my-api-resource"
    echo "  $0 alice create my-app api org-2 my-api-resource payload.json"
    echo "  $0 bob get my-service web org-1 my-resource"
    echo "  $0 charlie update my-service web org-1 my-resource update-payload.json"
    echo "  $0 dave delete my-service web org-1 my-resource"
}

# Function to make HTTP request
make_request() {
    local method=$1
    local url=$2
    local user_id=$3
    local payload_file=$4
    
    print_info "Making $method request to: $url"
    print_info "User ID: $user_id"
    
    # Build curl command
    local curl_cmd="curl -s -w \"\\n%{http_code}\\n\" -X $method"
    
    # Add user header if provided
    if [ ! -z "$user_id" ]; then
        curl_cmd="$curl_cmd -H \"X-User-Id: $user_id\""
    fi
    
    # Add content type header for POST/PUT
    if [ "$method" = "POST" ] || [ "$method" = "PUT" ]; then
        curl_cmd="$curl_cmd -H \"Content-Type: $CONTENT_TYPE\""
    fi
    
    # Add payload if provided
    if [ ! -z "$payload_file" ] && [ -f "$payload_file" ]; then
        print_info "Using payload from: $payload_file"
        curl_cmd="$curl_cmd -d @$payload_file"
    elif [ "$method" = "POST" ] || [ "$method" = "PUT" ]; then
        # Default payload for POST/PUT requests
        local default_payload="{\"description\": \"Test resource created via API\", \"tags\": [\"test\", \"api\"]}"
        curl_cmd="$curl_cmd -d '$default_payload'"
    fi
    
    # Add URL
    curl_cmd="$curl_cmd \"$url\""
    
    print_info "Executing: $curl_cmd"
    echo ""
    
    # Execute the request and capture output
    local response=$(eval $curl_cmd)
    local http_code=$(echo "$response" | tail -n1)
    local body=$(echo "$response" | head -n -1)
    
    # Print response
    echo "Response Body:"
    echo "$body" | jq . 2>/dev/null || echo "$body"
    echo ""
    echo "HTTP Status Code: $http_code"
    
    # Check status code
    if [ "$http_code" -ge 200 ] && [ "$http_code" -lt 300 ]; then
        print_success "Request successful!"
    elif [ "$http_code" -ge 400 ] && [ "$http_code" -lt 500 ]; then
        print_error "Client error (4xx)"
    elif [ "$http_code" -ge 500 ]; then
        print_error "Server error (5xx)"
    else
        print_warning "Unexpected status code: $http_code"
    fi
}

# Function to call health endpoint
call_health() {
    local url="$BASE_URL/health"
    make_request "GET" "$url" "" ""
}

# Function to call root endpoint
call_root() {
    local url="$BASE_URL/"
    make_request "GET" "$url" "" ""
}

# Function to call resource endpoints
call_resource() {
    local action=$1
    local user_id=$2
    local service_name=${3:-$DEFAULT_SERVICE_NAME}
    local service_type=${4:-$DEFAULT_SERVICE_TYPE}
    local org_id=${5:-$DEFAULT_ORG_ID}
    local resource_name=${6:-$DEFAULT_RESOURCE_NAME}
    local payload_file=$7
    
    local url="$BASE_URL/api/resource/$service_name/$service_type/$org_id/$resource_name"
    
    case $action in
        "create")
            make_request "POST" "$url" "$user_id" "$payload_file"
            ;;
        "get")
            make_request "GET" "$url" "$user_id" ""
            ;;
        "update")
            make_request "PUT" "$url" "$user_id" "$payload_file"
            ;;
        "delete")
            make_request "DELETE" "$url" "$user_id" ""
            ;;
        *)
            print_error "Unknown action: $action"
            show_usage
            exit 1
            ;;
    esac
}

# Main script logic
main() {
    # Check if curl is installed
    if ! command -v curl &> /dev/null; then
        print_error "curl is required but not installed. Please install curl."
        exit 1
    fi
    
    # Check if jq is installed (optional, for pretty JSON formatting)
    if ! command -v jq &> /dev/null; then
        print_warning "jq is not installed. JSON responses will not be formatted."
    fi
    
    # Check minimum arguments
    if [ $# -lt 2 ]; then
        print_error "Insufficient arguments"
        show_usage
        exit 1
    fi
    
    local user_id=$1
    local action=$2
    
    print_info "OpenFGA Demo API Test Script"
    print_info "=============================="
    
    case $action in
        "health")
            call_health
            ;;
        "root")
            call_root
            ;;
        "create"|"get"|"update"|"delete")
            call_resource "$action" "$user_id" "$3" "$4" "$5" "$6" "$7"
            ;;
        *)
            print_error "Unknown action: $action"
            show_usage
            exit 1
            ;;
    esac
}

# Run the main function with all arguments
main "$@"
