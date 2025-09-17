# Selective Sharing Guide: Parent → Multiple Child Organizations

## Overview

This guide demonstrates how to implement selective sharing where a parent organization (Org A) can share different resources with different child organizations (Org B and Org C) using OpenFGA's group-based authorization model.

## Organizational Structure

```
Org A (system)
├── Org B (partner)
└── Org C (partner_child)
```

**Users**:
- **Org A**: anne, bob, alice (system administrators)
- **Org B**: carl, diana (partner organization members)  
- **Org C**: emily, frank (partner_child organization members)

## Selective Sharing Strategy

### Three Sharing Groups

1. **`group:share_with_b_only`** - Resources shared ONLY with Org B
2. **`group:share_with_c_only`** - Resources shared ONLY with Org C
3. **`group:share_with_b_and_c`** - Resources shared with BOTH Org B and Org C

### Group Memberships

```yaml
# Org B members in selective sharing groups
- user: user:carl
  relation: member
  object: group:share_with_b_only
- user: user:carl
  relation: member  
  object: group:share_with_b_and_c

# Org C members in selective sharing groups
- user: user:emily
  relation: member
  object: group:share_with_c_only
- user: user:emily
  relation: member
  object: group:share_with_b_and_c

# (Similar for diana and frank)
```

## Sharing Scenarios

### Scenario 1: Share with Org B Only

**Resources shared ONLY with Org B (partner)**:

#### Full Service Sharing
```yaml
# Entire connector service → Only Org B
- user: group:share_with_b_only#member
  relation: shared_to_child_orgs
  object: service:connector
```

**Result**: Carl and Diana can access ALL connector resources:
- `resource:connector/s3/101`
- `resource:connector/mysql/301` 
- `resource:connector/gcs/401`
- `resource:connector/document/601`

Emily and Frank (Org C) **CANNOT** access any connector resources.

#### Service-Type Sharing
```yaml
# Analytics dashboard service type → Only Org B
- user: group:share_with_b_only#member
  relation: shared_to_child_orgs
  object: service_type:analytics/dashboard
```

**Result**: Carl and Diana can access:
- `resource:analytics/dashboard/main`

Emily and Frank **CANNOT** access analytics dashboard.

#### Individual Resource Sharing
```yaml
# Security audit logs → Only Org B
- user: group:share_with_b_only#member
  relation: shared_to_child_orgs
  object: resource:security/audit/logs
```

**Result**: Carl and Diana can access:
- `resource:security/audit/logs`

Emily and Frank **CANNOT** access security audit logs.

### Scenario 2: Share with Org C Only

**Resources shared ONLY with Org C (partner_child)**:

#### Service-Type Sharing
```yaml
# Analytics reports → Only Org C
- user: group:share_with_c_only#member
  relation: shared_to_child_orgs
  object: service_type:analytics/reports
```

**Result**: Emily and Frank can access:
- `resource:analytics/reports/monthly`

Carl and Diana (Org B) **CANNOT** access analytics reports.

#### Individual Resource Sharing
```yaml
# Security compliance reports → Only Org C  
- user: group:share_with_c_only#member
  relation: shared_to_child_orgs
  object: resource:security/compliance/reports

# Monitoring metrics → Only Org C
- user: group:share_with_c_only#member
  relation: shared_to_child_orgs
  object: service_type:monitoring/metrics
```

**Result**: Emily and Frank can access:
- `resource:security/compliance/reports`
- `resource:monitoring/metrics/cpu`

Carl and Diana **CANNOT** access these resources.

### Scenario 3: Share with Both Org B and Org C

**Resources shared with BOTH child organizations**:

#### Full Service Sharing
```yaml
# Security service → Both Org B and Org C
- user: group:share_with_b_and_c#member
  relation: shared_to_child_orgs
  object: service:security
```

**Result**: ALL child org members (Carl, Diana, Emily, Frank) can access:
- `resource:security/audit/logs`
- `resource:security/compliance/reports`

#### Individual Resource Sharing
```yaml
# Monitoring alerts → Both Org B and Org C
- user: group:share_with_b_and_c#member
  relation: shared_to_child_orgs
  object: resource:monitoring/alerts/critical

# Analytics monthly reports → Both Org B and Org C
- user: group:share_with_b_and_c#member
  relation: shared_to_child_orgs
  object: resource:analytics/reports/monthly
```

**Result**: ALL child org members can access:
- `resource:monitoring/alerts/critical`
- `resource:analytics/reports/monthly`

## Complete Access Matrix

| Resource/Service | Org A (Owner) | Org B Only | Org C Only | Both B & C |
|------------------|---------------|------------|------------|------------|
| **service:connector** | ✅ | ✅ | ❌ | ❌ |
| **service:analytics** | ✅ | ❌ | ❌ | ❌ |
| **service:security** | ✅ | ❌ | ❌ | ✅ |
| **service:monitoring** | ✅ | ❌ | ❌ | ❌ |
| **service_type:analytics/dashboard** | ✅ | ✅ | ❌ | ❌ |
| **service_type:analytics/reports** | ✅ | ❌ | ✅ | ❌ |
| **service_type:monitoring/metrics** | ✅ | ❌ | ✅ | ❌ |
| **resource:security/audit/logs** | ✅ | ✅ | ✅ | ✅ |
| **resource:security/compliance/reports** | ✅ | ❌ | ✅ | ✅ |
| **resource:monitoring/alerts/critical** | ✅ | ❌ | ❌ | ✅ |
| **resource:analytics/reports/monthly** | ✅ | ❌ | ❌ | ✅ |

## User Access Summary

### Carl (Org B Member)
**Can Access**:
- ✅ All connector resources (service sharing)
- ✅ Analytics dashboard (service-type sharing)
- ✅ Security audit logs (individual resource)
- ✅ All security resources (service sharing to both)
- ✅ Monitoring alerts (individual resource to both)
- ✅ Analytics monthly reports (individual resource to both)

**Cannot Access**:
- ❌ Analytics reports service type (Org C only)
- ❌ Security compliance reports (Org C only)
- ❌ Monitoring metrics (Org C only)

### Emily (Org C Member)  
**Can Access**:
- ✅ Analytics reports (service-type sharing)
- ✅ Security compliance reports (individual resource)
- ✅ Monitoring metrics (service-type sharing)
- ✅ All security resources (service sharing to both)
- ✅ Monitoring alerts (individual resource to both)
- ✅ Analytics monthly reports (individual resource to both)

**Cannot Access**:
- ❌ All connector resources (Org B only)
- ❌ Analytics dashboard (Org B only)
- ❌ Security audit logs (Org B only)

## Implementation Benefits

### 1. **Granular Control**
- Share at service, service-type, or individual resource level
- Different sharing patterns for different child organizations

### 2. **Scalable Group Management**
- Add/remove users from sharing groups
- No need to update individual resource permissions

### 3. **Flexible Sharing Patterns**
- Exclusive sharing (A→B only, A→C only)
- Inclusive sharing (A→B+C)
- Mixed sharing patterns

### 4. **Inheritance Support**
- Service sharing automatically includes all service types and resources
- Service-type sharing includes all resources of that type

## Testing Scenarios

### Test 1: Verify Org B Exclusive Access
```bash
# Carl should have access to connector service
curl -X POST "http://localhost:8080/stores/${STORE_ID}/check" \
  -d '{"tuple_key": {"user": "user:carl", "relation": "viewer", "object": "service:connector"}}'

# Emily should NOT have access to connector service
curl -X POST "http://localhost:8080/stores/${STORE_ID}/check" \
  -d '{"tuple_key": {"user": "user:emily", "relation": "viewer", "object": "service:connector"}}'
```

### Test 2: Verify Org C Exclusive Access
```bash
# Emily should have access to analytics reports
curl -X POST "http://localhost:8080/stores/${STORE_ID}/check" \
  -d '{"tuple_key": {"user": "user:emily", "relation": "viewer", "object": "service_type:analytics/reports"}}'

# Carl should NOT have access to analytics reports
curl -X POST "http://localhost:8080/stores/${STORE_ID}/check" \
  -d '{"tuple_key": {"user": "user:carl", "relation": "viewer", "object": "service_type:analytics/reports"}}'
```

### Test 3: Verify Both Org Access
```bash
# Both Carl and Emily should have access to security service
curl -X POST "http://localhost:8080/stores/${STORE_ID}/check" \
  -d '{"tuple_key": {"user": "user:carl", "relation": "viewer", "object": "service:security"}}'

curl -X POST "http://localhost:8080/stores/${STORE_ID}/check" \
  -d '{"tuple_key": {"user": "user:emily", "relation": "viewer", "object": "service:security"}}'
```

## Advanced Patterns

### Dynamic Group Management
```yaml
# Add new user to specific sharing group
- user: user:new_user
  relation: member
  object: group:share_with_b_only
```

### Conditional Sharing
```yaml
# Share different resources based on conditions
# (Requires OpenFGA conditions feature)
- user: group:conditional_share#member
  relation: shared_to_child_orgs
  object: resource:sensitive/data
  condition: "user.department == 'engineering'"
```

### Hierarchical Sharing
```yaml
# Share parent service but exclude specific service types
- user: group:partial_share#member
  relation: shared_to_child_orgs
  object: service:analytics

# Explicitly deny specific service type
- user: group:partial_share#member
  relation: denied_access
  object: service_type:analytics/sensitive
```

## Best Practices

1. **Use Descriptive Group Names**: `share_with_finance_only`, `share_with_engineering_and_ops`
2. **Document Sharing Policies**: Maintain clear documentation of what gets shared where
3. **Regular Access Reviews**: Periodically review group memberships and sharing rules
4. **Principle of Least Privilege**: Only share what's necessary for each organization
5. **Audit Logging**: Track resource access patterns and sharing changes

This selective sharing model provides the flexibility to implement complex sharing scenarios while maintaining clear authorization boundaries between different child organizations.
