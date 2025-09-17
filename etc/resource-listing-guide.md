# Resource Listing Guide for OpenFGA

## Overview

Resource listing and discovery in OpenFGA requires special consideration since OpenFGA is designed primarily for authorization checks, not resource enumeration. This guide provides multiple approaches to solve the resource listing challenge.

## Approaches for Resource Listing

### 1. OpenFGA ListObjects API (Recommended)

The `ListObjects` API endpoint allows you to discover what objects a user has access to for a given relation.

#### Advantages:
- ✅ Native OpenFGA functionality
- ✅ Always up-to-date with current permissions
- ✅ Respects all authorization rules and inheritance
- ✅ No additional infrastructure needed

#### Disadvantages:
- ⚠️ Experimental feature (as of 2024)
- ⚠️ Potential performance concerns for large datasets
- ⚠️ Limited filtering and pagination options

#### Implementation:

**API Endpoint**: `GET /api/list-objects`

**Query Parameters**:
- `object_type` (optional): Type of objects to list (`service`, `service_type`, `resource`)
- `relation` (optional): Relation to check (`viewer`, `editor`, `admin`)

**Example Requests**:

```bash
# List all services user can view
curl -H "Authorization: Bearer <token>" \
  "http://localhost:3000/api/list-objects?object_type=service&relation=viewer"

# List all resources user can edit
curl -H "Authorization: Bearer <token>" \
  "http://localhost:3000/api/list-objects?object_type=resource&relation=editor"

# List all service types user has admin access to
curl -H "Authorization: Bearer <token>" \
  "http://localhost:3000/api/list-objects?object_type=service_type&relation=admin"
```

**Example Response**:
```json
{
  "objects": [
    "service:connector",
    "service:analytics"
  ],
  "total_count": 2,
  "object_type": "service",
  "relation": "viewer"
}
```

### 2. Comprehensive Shared Resources API

This approach queries multiple object types and relations to provide a complete view of shared resources.

#### API Endpoint: `GET /api/shared-resources`

**Example Request**:
```bash
curl -H "Authorization: Bearer <token>" \
  "http://localhost:3000/api/shared-resources"
```

**Example Response**:
```json
{
  "services": [
    {
      "id": "service:connector",
      "name": "connector",
      "shared_via": "parent_organization",
      "permissions": ["viewer", "editor"]
    }
  ],
  "service_types": [
    {
      "id": "service_type:connector/s3",
      "service_name": "connector",
      "service_type": "s3",
      "shared_via": "parent_organization",
      "permissions": ["viewer"]
    }
  ],
  "resources": [
    {
      "id": "resource:connector/s3/101",
      "service_name": "connector",
      "service_type": "s3",
      "resource_name": "101",
      "shared_via": "parent_organization",
      "permissions": ["viewer", "editor"]
    }
  ]
}
```

### 3. Application-Level Caching (For High Performance)

For production systems with performance requirements, consider maintaining an application-level cache.

#### Database Schema Example:

```sql
-- Resource registry table
CREATE TABLE resource_registry (
    id SERIAL PRIMARY KEY,
    object_id VARCHAR(255) NOT NULL UNIQUE,
    object_type VARCHAR(50) NOT NULL,
    service_name VARCHAR(100),
    service_type VARCHAR(100),
    resource_name VARCHAR(100),
    parent_org VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Sharing permissions cache
CREATE TABLE sharing_permissions (
    id SERIAL PRIMARY KEY,
    object_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255),
    group_id VARCHAR(255),
    relation VARCHAR(50) NOT NULL,
    granted_via VARCHAR(100), -- 'direct', 'group', 'parent_org'
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_user_relation (user_id, relation),
    INDEX idx_group_relation (group_id, relation),
    INDEX idx_object_relation (object_id, relation)
);
```

#### Cache Update Strategy:

1. **Write-Through**: Update cache when permissions change
2. **Periodic Refresh**: Sync with OpenFGA periodically
3. **Lazy Loading**: Update cache on first access

## Listing Scenarios and Examples

### Scenario 1: List All Shared Services

**Goal**: Find all services shared from parent organizations to child organizations.

**Method 1 - ListObjects API**:
```bash
# For user in child organization
curl -H "Authorization: Bearer carl_token" \
  "http://localhost:3000/api/list-objects?object_type=service&relation=viewer"
```

**Method 2 - Direct OpenFGA API**:
```bash
curl -X POST "http://localhost:8080/stores/${STORE_ID}/list-objects" \
  -H "Content-Type: application/json" \
  -d '{
    "type": "service",
    "relation": "viewer", 
    "user": "user:carl"
  }'
```

### Scenario 2: List Service Types Available to User

**Goal**: Find all service types a user can access (both owned and shared).

```bash
curl -H "Authorization: Bearer emily_token" \
  "http://localhost:3000/api/list-objects?object_type=service_type&relation=viewer"
```

**Expected Result for Emily (partner_child org member)**:
- Should see `service_type:connector/s3` (shared from system org)
- Should NOT see `service_type:connector/mysql` (not shared)

### Scenario 3: List All Resources with Different Permission Levels

**Goal**: Find resources with different access levels.

```bash
# Resources user can view
curl -H "Authorization: Bearer diana_token" \
  "http://localhost:3000/api/list-objects?object_type=resource&relation=viewer"

# Resources user can edit  
curl -H "Authorization: Bearer diana_token" \
  "http://localhost:3000/api/list-objects?object_type=resource&relation=editor"

# Resources user can admin
curl -H "Authorization: Bearer diana_token" \
  "http://localhost:3000/api/list-objects?object_type=resource&relation=admin"
```

### Scenario 4: Comprehensive Resource Discovery

**Goal**: Get complete view of all shared resources.

```bash
curl -H "Authorization: Bearer carl_token" \
  "http://localhost:3000/api/shared-resources"
```

## Testing Resource Listing

### Setup Test Data

1. **Start OpenFGA and load the model**:
```bash
./scripts/start-openfga.sh
./scripts/demo.sh
```

2. **Test with different users**:

**Carl (partner org member)** - Should see:
- Full connector service (via `group:partner_share`)
- All connector resources (s3/101, mysql/301, gcs/401, document/601)

**Emily (partner_child org member)** - Should see:
- Only s3 service type (via `group:child_share`)
- Only s3/101 resource

**Diana (partner org member)** - Should see:
- Full connector service (via `group:partner_share`)
- Plus direct editor access to gcs/401

### Test Commands

```bash
# Test Carl's access (should see full service sharing)
curl -H "Authorization: Bearer carl_token" \
  "http://localhost:3000/api/shared-resources" | jq .

# Test Emily's access (should see only s3 service type)
curl -H "Authorization: Bearer emily_token" \
  "http://localhost:3000/api/list-objects?object_type=service_type&relation=viewer" | jq .

# Test specific resource access
curl -H "Authorization: Bearer carl_token" \
  "http://localhost:3000/api/list-objects?object_type=resource&relation=viewer" | jq .
```

## Performance Considerations

### OpenFGA ListObjects Performance

- **Small datasets** (< 1000 objects): Usually fast
- **Medium datasets** (1K-10K objects): May have latency
- **Large datasets** (> 10K objects): Consider caching

### Optimization Strategies

1. **Pagination**: Implement client-side pagination
2. **Filtering**: Filter by object type and relation
3. **Caching**: Cache results for frequently accessed data
4. **Batch Operations**: Group multiple list operations
5. **Background Sync**: Periodically sync with cache

## Best Practices

1. **Start with ListObjects**: Use native OpenFGA functionality first
2. **Monitor Performance**: Track response times and optimize as needed
3. **Implement Caching**: Add caching layer for production systems
4. **Error Handling**: Handle API failures gracefully
5. **Security**: Ensure listing doesn't leak unauthorized information
6. **Audit Logging**: Log resource access patterns

## Alternative Approaches

### 1. Read API with Known Object IDs

If you maintain a registry of object IDs, you can use the Check API:

```bash
# Check if user has access to known resources
for resource in $(cat known_resources.txt); do
  curl -X POST "http://localhost:8080/stores/${STORE_ID}/check" \
    -d "{\"tuple_key\": {\"user\": \"user:carl\", \"relation\": \"viewer\", \"object\": \"$resource\"}}"
done
```

### 2. Tuple Enumeration (Not Recommended)

Reading all tuples and filtering client-side is possible but not recommended for production due to performance and security concerns.

## Troubleshooting

### Common Issues:

1. **Empty Results**: Check if authorization model is loaded correctly
2. **Permission Denied**: Verify user authentication and group memberships
3. **Performance Issues**: Consider implementing caching or pagination
4. **Inconsistent Results**: Ensure tuples are properly loaded

### Debug Commands:

```bash
# Check if user is properly authenticated
curl -H "Authorization: Bearer <token>" "http://localhost:3000/health"

# Verify OpenFGA store and model
curl "http://localhost:8080/stores/${STORE_ID}"
curl "http://localhost:8080/stores/${STORE_ID}/authorization-models"
```

This comprehensive approach gives you multiple options for resource listing depending on your performance, scalability, and complexity requirements.
