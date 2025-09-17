# Resource Sharing Guide for OpenFGA Demo

## Overview

This enhanced OpenFGA model supports hierarchical resource sharing from parent organizations to child organizations through groups. The system provides three levels of granular sharing:

1. **Full Service Sharing** - Share all resources under a service
2. **Service-Type Sharing** - Share all resources of a specific service-type  
3. **Individual Resource Sharing** - Share specific resources

## Model Architecture

### New Types Added

#### `service`
- Represents a logical grouping of service types (e.g., "connector")
- Relations:
  - `parent_org`: The organization that owns the service
  - `shared_to_child_orgs`: Groups that can access this service from child orgs
  - `admin`, `editor`, `viewer`: Permission levels

#### `service_type` 
- Represents a specific type within a service (e.g., "connector/s3", "connector/mysql")
- Relations:
  - `parent_service`: The service this type belongs to
  - `parent_org`: The organization that owns this service type
  - `shared_to_child_orgs`: Groups that can access this service type from child orgs
  - `admin`, `editor`, `viewer`: Permission levels with inheritance from parent service

#### Enhanced `resource`
- Now connects to both service and service_type for hierarchical permissions
- Relations:
  - `parent_service_type`: The service type this resource belongs to
  - `parent_service`: The service this resource belongs to
  - `parent_org`: The organization that owns this resource
  - `shared_to_child_orgs`: Groups that can access this resource from child orgs
  - `owner`, `admin`, `editor`, `viewer`: Permission levels with inheritance

## Sharing Mechanisms

### 1. Full Service Sharing

**Use Case**: Share an entire service (all service types and resources) from parent to child organization.

**Example**: System org shares the entire "connector" service with partner org
```yaml
- user: group:partner_share#member
  relation: shared_to_child_orgs
  object: service:connector
```

**Result**: Members of `group:partner_share` from child organizations can access ALL resources under the connector service (s3, mysql, gcs, document, etc.)

### 2. Service-Type Sharing

**Use Case**: Share only specific service types from parent to child organization.

**Example**: System org shares only "s3" connector type with partner_child org
```yaml
- user: group:child_share#member
  relation: shared_to_child_orgs
  object: service_type:connector/s3
```

**Result**: Members of `group:child_share` from child organizations can access ALL s3 connector resources, but not mysql, gcs, or document resources.

### 3. Individual Resource Sharing

**Use Case**: Share specific resources from parent to child organization.

**Example**: System org shares a specific document with partner org
```yaml
- user: group:partner_share#member
  relation: shared_to_child_orgs
  object: resource:connector/document/601
```

**Result**: Members of `group:partner_share` from child organizations can access only that specific document resource.

## Permission Inheritance

The model implements cascading permissions:

1. **Service Level**: `shared_to_child_orgs` on a service grants access to all service types and resources under it
2. **Service-Type Level**: `shared_to_child_orgs` on a service type grants access to all resources of that type
3. **Resource Level**: `shared_to_child_orgs` on a resource grants access to that specific resource

## Example Scenarios

### Scenario 1: Complete Service Access
- **Setup**: Partner org is child of System org
- **Sharing**: System shares entire "connector" service via `group:partner_share`
- **Members**: Carl and Diana are in `group:partner_share`
- **Result**: Carl and Diana can access ALL connector resources (s3/101, mysql/301, gcs/401, document/601)

### Scenario 2: Limited Service-Type Access  
- **Setup**: Partner_child org is child of Partner org (grandchild of System)
- **Sharing**: System shares only "s3" service type via `group:child_share`
- **Members**: Emily and Frank are in `group:child_share`
- **Result**: Emily and Frank can access ONLY s3 resources (s3/101), not mysql, gcs, or document

### Scenario 3: Specific Resource Access
- **Setup**: Partner org is child of System org
- **Sharing**: System shares specific document/601 resource via `group:partner_share`
- **Members**: Carl and Diana are in `group:partner_share`
- **Result**: Carl and Diana can access ONLY document/601, not other resources

## Resource Naming Convention

Resources follow the pattern: `resource:{service_name}/{service_type}/{resource_id}`

Examples:
- `resource:connector/s3/101`
- `resource:connector/mysql/301`
- `resource:connector/gcs/401`
- `resource:connector/document/601`

## Group-Based Sharing Benefits

1. **Scalability**: Add/remove users from groups instead of individual resource permissions
2. **Maintainability**: Centralized permission management through groups
3. **Flexibility**: Different groups can have different sharing scopes
4. **Security**: Fine-grained control over what gets shared and with whom

## Implementation Notes

- All sharing is done via groups (`group#member`) rather than individual users
- Child organizations inherit permissions through the `shared_to_child_orgs` relation
- The model supports multi-level organizational hierarchies
- Permission levels (admin, editor, viewer) are preserved across sharing boundaries
- Users must be members of the appropriate sharing group to access shared resources

## Testing Sharing Scenarios

You can test these scenarios using the OpenFGA API:

```bash
# Test if Carl (partner org member) can view s3/101 (shared via full service sharing)
curl -X POST http://localhost:8080/stores/{store_id}/check \
  -H "Content-Type: application/json" \
  -d '{
    "tuple_key": {
      "user": "user:carl",
      "relation": "viewer", 
      "object": "resource:connector/s3/101"
    }
  }'

# Test if Emily (partner_child org member) can view mysql/301 (should be denied - only s3 shared)
curl -X POST http://localhost:8080/stores/{store_id}/check \
  -H "Content-Type: application/json" \
  -d '{
    "tuple_key": {
      "user": "user:emily",
      "relation": "viewer",
      "object": "resource:connector/mysql/301"
    }
  }'
```
