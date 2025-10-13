# Multi-Tenancy Authorization Model

This directory contains the OpenFGA authorization model for multi-tenant resource access control with fine-grained CRUD permissions.

## 📁 File Structure

### Core Model
- **`model.fga`** - The OpenFGA authorization model definition
  - Organisation-scoped groups and roles
  - Resource-level permissions with CRUD hierarchy
  - Support for resource-specific access overrides

### Tuple Files (Reusable Data)
- **`tuples-org-users-groups.yaml`** - Organisations, users, and groups
  - Organisation A1 and A2 setup
  - Groups (platform, devops, solution under engineering parent)
  - Users (3 per team, following pattern: `A1-plt-u1`, etc.)
  - Group assignments to organisations

- **`tuples-resources.yaml`** - Basic resource definitions
  - Resources for Organisation A1 and A2
  - Resource-to-organisation relationships

### Test Files

Test files import tuple files using `tuple_files` to avoid duplication:

- **`user-groups.yaml`** - Tests for user groups and organisation structure
  - Organisation ownership and permissions
  - Group membership (including nested groups)
  - Multi-tenancy isolation
  - Group hierarchy validation

- **`res-curd.yaml`** - Comprehensive CRUD permission tests
  - CRUD permission hierarchy (delete > update > get > list)
  - Resource-specific permissions (same org, different access)
  - Multi-tenancy CRUD isolation
  - Delete permission restrictions
  - GET vs LIST (credentials access control)
  - CREATE permission verification

## 🎯 Permission Hierarchy

### Organisation Roles
```
owner → admin → resource_manager → editor → viewer
```

### CRUD Operations
```
can_delete (most restrictive)
    ↓
can_update
    ↓
can_get (shows credentials)
    ↓
can_list (shows basic info only)
```

## 🚀 Running Tests

### Run All Tests
```bash
fga model test --tests "fga-models/multi-tenancy/*.yaml"
```

### Run Specific Test File
```bash
# Test user groups and organisations
fga model test --tests fga-models/multi-tenancy/user-groups.yaml

# Test CRUD operations
fga model test --tests fga-models/multi-tenancy/res-curd.yaml
```

### Verbose Output
```bash
fga model test --tests "fga-models/multi-tenancy/*.yaml" --verbose
```

## 📋 Test Coverage

### User Groups Tests
- ✅ Organisation ownership verification
- ✅ Group membership (direct and nested)
- ✅ Permission inheritance through groups
- ✅ Multi-tenancy isolation (cross-org access denial)
- ✅ Group hierarchy validation

### CRUD Tests
- ✅ Permission hierarchy enforcement
- ✅ Resource-specific permission overrides
- ✅ Multi-tenancy CRUD isolation
- ✅ Delete restrictions (owner/admin only)
- ✅ GET vs LIST access control
- ✅ CREATE permission verification

## 🔐 Key Features

### 1. Multi-Tenancy Isolation
- Groups and roles are scoped to organisations
- Resources belong to organisations
- No cross-organisation access possible

### 2. Fine-Grained Resource Permissions
Users in the same organisation can have different permissions on different resources:
```yaml
# Platform team: editor on most resources
# But only viewer on secret vault
- user: group:A1-platform#member
  object: resource:A1-secret-vault
  relation: viewer
```

### 3. Credential Protection
- **LIST**: Shows basic resource information
- **GET**: Shows full details including credentials (more restrictive)

### 4. Delete Protection
Only `owner` and `admin` can delete resources. Even `resource_manager` cannot delete.

## ⚠️ Important Notes

### Permission Inheritance Behavior

**Important:** Permissions granted at the organisation level **cannot be downgraded** at the resource level:

- ✅ If a user is `editor` at org level → they're `editor` on ALL resources in that org
- ✅ You can **UPGRADE** permissions on specific resources (e.g., `editor` → `resource_manager`)
- ❌ You **CANNOT DOWNGRADE** permissions (e.g., `editor` → `viewer` on specific resource)

**Example:**
```yaml
# User is editor via group
- user: group:eng#member
  object: organisation:A1
  relation: editor

# This does NOT restrict the user to viewer - they remain editor!
- user: group:eng#member
  object: resource:R1
  relation: viewer

# Result: User is BOTH editor (from org) AND viewer (from resource)
# Since editor > viewer, they have editor permissions
```

**To achieve resource-specific restrictions:**
- Create separate groups with different org-level permissions
- Use viewer-only groups for users who should only view resources
- Use resource-specific grants to UPGRADE viewer users to higher permissions on specific resources

### Application-Level Validation Required

The model **cannot enforce** that directly assigned users/groups/roles belong to the same organisation as the resource. Your application **MUST validate** this:

```rust
// Before writing: resource:R1#editor@group:G1#member
// Check that: group:G1#organisation@organisation:ORG exists
// Where ORG is the organisation that resource R1 belongs to
```

See comments in `model.fga` for detailed validation requirements.

### Tuple File Imports

Test files use `tuple_files` to import multiple tuple files. This allows:
- ✅ DRY (Don't Repeat Yourself) - define tuples once
- ✅ Modular organization - separate concerns (orgs/users vs resources)
- ✅ Consistency across tests
- ✅ Easy maintenance
- ✅ Focused test files (only test-specific tuples)

Example:
```yaml
tuple_files:
  - ./tuples-org-users-groups.yaml
  - ./tuples-resources.yaml
```

## 📊 Test Organization Structure

### Organisation A1
- **Owner**: `user:A1-owner`
- **Engineering Group**: `group:A1-eng` (contains all teams)
  - **Platform Team**: `A1-plt-u1`, `A1-plt-u2`, `A1-plt-u3`
  - **DevOps Team**: `A1-dev-u1`, `A1-dev-u2`, `A1-dev-u3`
  - **Solution Team**: `A1-sol-u1`, `A1-sol-u2`, `A1-sol-u3`
- **Org Permission**: Engineering group assigned as `editor`

### Organisation A2
- **Owner**: `user:A2-owner`
- **Engineering Group**: `group:A2-eng` (contains all teams)
  - **Platform Team**: `A2-plt-u1`, `A2-plt-u2`, `A2-plt-u3`
  - **DevOps Team**: `A2-dev-u1`, `A2-dev-u2`, `A2-dev-u3`
  - **Solution Team**: `A2-sol-u1`, `A2-sol-u2`, `A2-sol-u3`
- **Org Permission**: Engineering group assigned as `editor`

## 🔄 Adding New Tests

To create a new test file that reuses existing tuples:

```yaml
name: My New Test
model_file: ./model.fga

# Import shared tuples
tuple_file: ./shared-tuples.yaml

# Add test-specific tuples
tuples:
  - user: organisation:A1
    object: resource:A1-my-new-resource
    relation: organisation

tests:
  - name: My Test Case
    check:
      - user: user:A1-plt-u1
        object: resource:A1-my-new-resource
        assertions:
          can_update: true
```

## 📚 Related Documentation

- [OpenFGA Documentation](https://openfga.dev/docs)
- [Store File Format](https://github.com/openfga/cli/blob/main/docs/STORE_FILE.md)
- [Testing Models](https://openfga.dev/docs/modeling/testing)

