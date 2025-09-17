# API Testing Scripts

This directory contains scripts and utilities for testing the OpenFGA Demo API.

## Files

- `api-test.sh` - Main API testing script
- `payloads/` - Directory containing sample JSON payloads
  - `create-resource.json` - Sample payload for creating resources
  - `update-resource.json` - Sample payload for updating resources

## Prerequisites

- `curl` - Required for making HTTP requests
- `jq` - Optional, for pretty JSON formatting in responses

## Usage

The main script `api-test.sh` allows you to test all API endpoints with a simple command-line interface.

### Basic Syntax

```bash
./api-test.sh <user-id> <action> [service-name] [service-type] [org-id] [resource-name] [payload-file]
```

### Available Actions

- `health` - Check API health (no authentication required)
- `root` - Get root endpoint (no authentication required)
- `create` - Create a resource (POST)
- `get` - Get a resource (GET)
- `update` - Update a resource (PUT)
- `delete` - Delete a resource (DELETE)

### Parameters

- **user-id** - User ID to send in `X-User-Id` header (required for resource operations)
- **service-name** - Service name (default: `my-service`)
- **service-type** - Service type (default: `web`)
- **org-id** - Organization ID (default: `org-1`)
- **resource-name** - Resource name (default: `my-resource`)
- **payload-file** - JSON file for POST/PUT requests (optional)

## Examples

### Public Endpoints (No Authentication)

```bash
# Check API health
./api-test.sh alice health

# Get root endpoint
./api-test.sh alice root
```

### Resource Operations with Default Parameters

```bash
# Create a resource with default parameters (my-service/web/org-1/my-resource)
./api-test.sh alice create

# Get a resource with default parameters
./api-test.sh alice get

# Update a resource with default parameters
./api-test.sh alice update

# Delete a resource with default parameters
./api-test.sh alice delete
```

### Resource Operations with Custom Parameters

```bash
# Create a custom resource
./api-test.sh bob create my-app api org-2 my-api-resource

# Get a specific resource
./api-test.sh charlie get payment-service microservice org-3 payment-processor

# Update with custom payload
./api-test.sh dave update my-service web org-1 my-resource payloads/update-resource.json

# Delete a specific resource
./api-test.sh eve delete my-app api org-2 my-api-resource
```

### Using Custom Payloads

```bash
# Create resource with custom payload
./api-test.sh alice create my-service web org-1 my-resource payloads/create-resource.json

# Update resource with custom payload
./api-test.sh alice update my-service web org-1 my-resource payloads/update-resource.json
```

## Testing Different Users and Permissions

Based on the OpenFGA authorization model, you can test different permission scenarios:

```bash
# Test as different users
./api-test.sh alice get my-service web org-1 my-resource
./api-test.sh bob create my-service web org-1 new-resource
./api-test.sh charlie update my-service web org-1 my-resource
./api-test.sh dave delete my-service web org-1 my-resource

# Test with different organizations
./api-test.sh alice get my-service web org-1 resource-1
./api-test.sh alice get my-service web org-2 resource-2
```

## Expected Responses

### Successful Requests

- **200 OK** - GET, PUT, DELETE operations
- **201 Created** - POST operations

### Authentication Errors

- **401 Unauthorized** - Missing `X-User-Id` header
- **400 Bad Request** - Invalid or empty `X-User-Id` header

### Authorization Errors

- **403 Forbidden** - User doesn't have required permissions

### Sample Success Response

```json
{
  "message": "Resource created successfully",
  "resource_id": "my-service/web/org-1/my-resource"
}
```

### Sample Error Response

```json
{
  "error": "Permission denied",
  "message": "You do not have permission to create this resource"
}
```

## Troubleshooting

1. **Script not executable**: Run `chmod +x api-test.sh`
2. **curl not found**: Install curl using your package manager
3. **Connection refused**: Make sure the API server is running on `localhost:3000`
4. **Permission denied**: Check that the user has the required permissions in OpenFGA
5. **Invalid JSON**: Validate your payload files using `jq . < payload-file.json`

## Customization

You can modify the script to:

- Change the default `BASE_URL` (currently `http://localhost:3000`)
- Add new actions or endpoints
- Modify default parameter values
- Add additional headers or authentication methods

## Integration with CI/CD

The script returns appropriate exit codes and can be used in automated testing:

```bash
# Run in CI/CD pipeline
./api-test.sh alice health || exit 1
./api-test.sh alice create || exit 1
./api-test.sh alice get || exit 1
```
