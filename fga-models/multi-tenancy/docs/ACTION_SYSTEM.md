# Action System - Multi-Tenancy Authorization Model

## Overview

The action system enables **dynamic, per-resource custom actions** with multi-tenancy support. Actions like `generate-pdf`, `generate-analytics`, `call-proxy`, `generate-signed-url`, etc., can be defined arbitrarily per resource.

## Key Features

✅ **Organization-scoped** - Actions belong to organizations for multi-tenancy isolation  
✅ **Resource-specific** - Each action is linked to a specific resource  
✅ **Default permissions** - Resource managers and editors can perform actions by default  
✅ **Custom performers** - Additional users, groups, or roles can be assigned per action  
✅ **Dynamic** - Action types can vary across different resources  

## Model Structure

### Action Type Definition

```fga
type action
    relations
        # Action belongs to an organisation (multi-tenancy)
        define organisation: [organisation]

        # Action is linked to a specific resource
        define resource: [resource]

        # Custom performers for this action
        define performer: [user, group#member, role#assignee]

        # Permission check for performing the action
        define can_perform_action: resource_manager from resource or editor from resource or performer
```

### How It Works

1. **Action → Resource Link**: Each action must be linked to a resource via the `resource` relation
2. **Default Permissions**: The action checks if the user is a `resource_manager` or `editor` of the linked resource
3. **Custom Performers**: Additional performers can be assigned directly to the action

## Tuple Structure

For an action `generate-pdf` on resource `database-1` in organization `acme`:

```yaml
# 1. Scope action to organization
- user: organisation:acme
  object: action:db1-generate-pdf
  relation: organisation

# 2. Link action to resource
- user: resource:database-1
  object: action:db1-generate-pdf
  relation: resource

# 3. Assign custom performers (optional)
- user: group:analytics-team#member
  object: action:db1-generate-pdf
  relation: performer

- user: user:alice
  object: action:db1-generate-pdf
  relation: performer

- user: role:report-generator#assignee
  object: action:db1-generate-pdf
  relation: performer
```

## Permission Hierarchy

For a given action, users can perform it through:

1. **Owner** → admin → resource_manager (via resource)
2. **Resource Manager** (org-level or resource-level)
3. **Editor** (org-level or resource-level)
4. **Custom Performer** (user, group member, or role assignee)

### Permission Flow Example

```
Check: user:alice can perform action:db1-generate-pdf?

1. Is alice a resource_manager of database-1? ✅ YES → ALLOW
   OR
2. Is alice an editor of database-1? ✅ YES → ALLOW
   OR
3. Is alice assigned as a performer of this action? ✅ YES → ALLOW

Otherwise → DENY
```

## Multi-Tenancy Isolation

### Critical Requirements

⚠️ **APPLICATION-LEVEL VALIDATION REQUIRED**

Before assigning performers to actions, your application **MUST** verify they belong to the same organization:

#### For Groups:
```
Before writing: action:A1#performer@group:analytics#member
Verify: group:analytics#organisation@organisation:ACME exists
```

#### For Users:
```
Before writing: action:A1#performer@user:alice
Verify: user:alice has viewer+ permission in organization ACME
Check: user:alice -> organisation:ACME#viewer
```

#### For Roles:
```
Before writing: action:A1#performer@role:reporter#assignee
Verify: role:reporter#organisation@organisation:ACME exists
```

**Failure to validate will break multi-tenancy isolation!**

## Usage Examples

### Example 1: PDF Generation with Group Access

```yaml
# Setup action
- user: organisation:acme
  object: action:invoice-generate-pdf
  relation: organisation

- user: resource:invoice-123
  object: action:invoice-generate-pdf
  relation: resource

# Assign analytics group
- user: group:analytics#member
  object: action:invoice-generate-pdf
  relation: performer

# Check permission
Check: user:bob can perform action:invoice-generate-pdf#can_perform_action?
Result: TRUE (if bob is in analytics group)
```

### Example 2: Signed URL with Role-Based Access

```yaml
# Setup action
- user: organisation:acme
  object: action:vault-signed-url
  relation: organisation

- user: resource:secret-vault
  object: action:vault-signed-url
  relation: resource

# Assign role
- user: role:url-generator#assignee
  object: action:vault-signed-url
  relation: performer

# Check permission
Check: user:alice can perform action:vault-signed-url#can_perform_action?
Result: TRUE (if alice is assignee of url-generator role OR editor of secret-vault)
```

### Example 3: Analytics with User-Specific Access

```yaml
# Setup action
- user: organisation:acme
  object: action:db-analytics
  relation: organisation

- user: resource:customer-db
  object: action:db-analytics
  relation: resource

# Assign specific user
- user: user:data-scientist
  object: action:db-analytics
  relation: performer

# Check permission
Check: user:data-scientist can perform action:db-analytics#can_perform_action?
Result: TRUE (explicitly assigned)

Check: user:random-user can perform action:db-analytics#can_perform_action?
Result: FALSE (unless they're editor/resource_manager of customer-db)
```

### Example 4: Action Without Custom Performers

```yaml
# Setup action (no custom performers)
- user: organisation:acme
  object: action:cache-export
  relation: organisation

- user: resource:redis-cache
  object: action:cache-export
  relation: resource

# No performer assignments

# Check permission
Check: user:editor can perform action:cache-export#can_perform_action?
Result: TRUE (if user is editor of redis-cache)

Check: user:viewer can perform action:cache-export#can_perform_action?
Result: FALSE (viewers cannot perform actions)
```

## Test Coverage

The test file `tests/actions.yaml` includes comprehensive tests for:

1. ✅ Default action permissions (resource managers & editors)
2. ✅ Custom action performers (users, groups, roles)
3. ✅ Multi-tenancy isolation
4. ✅ Viewer restrictions
5. ✅ Actions without custom performers
6. ✅ Resource-specific action permissions
7. ✅ Combined permissions (multiple assignment paths)
8. ✅ Action hierarchy through organization roles
9. ✅ Dynamic action names support

## Common Patterns

### Pattern 1: Team-Based Action Access
Assign entire teams to specific actions:
```yaml
- user: group:devops-team#member
  object: action:db-backup
  relation: performer
```

### Pattern 2: Temporary Action Access
Grant specific users temporary access to actions:
```yaml
- user: user:contractor-alice
  object: action:generate-report
  relation: performer
```

### Pattern 3: Role-Based Action Access
Use roles for consistent action permissions:
```yaml
- user: role:report-generator#assignee
  object: action:pdf-export
  relation: performer
```

## Best Practices

1. **Name actions clearly**: Use format `{resource-id}-{action-type}` (e.g., `db1-generate-pdf`)
2. **Default to org permissions**: Only add custom performers when needed
3. **Validate organization membership**: Always verify performers belong to the same org
4. **Document action types**: Maintain a registry of available action types per resource type
5. **Use groups over users**: Prefer group assignments for easier management
6. **Audit action usage**: Track who performs what actions for security

## Fixes Applied

### Issue 1: Incorrect Permission Location
**Problem**: `can_perform_action` was defined on `resource` type, not `action` type  
**Fix**: Moved to `action` type with proper resource lookups

### Issue 2: Redundant Tuples
**Problem**: Tests had both `action → resource` and `resource → action` links  
**Fix**: Removed redundant `resource.action` relation and tuples

### Issue 3: Model Cleanup
**Problem**: Unused `action` relation on resource type  
**Fix**: Removed unused relation from model

## Migration Guide

If you have existing actions using the old model:

1. **Remove redundant tuples**: Delete all `resource#action@action:*` tuples
2. **Keep action tuples**: Keep `action#resource@resource:*` tuples
3. **Update checks**: Change checks from `resource#can_perform_action` to `action#can_perform_action`

## Summary

The action system provides a flexible, multi-tenant way to manage custom operations on resources. By combining default permissions (resource_manager, editor) with custom performer assignments (users, groups, roles), you can implement fine-grained access control for any dynamic action type.

