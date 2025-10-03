# OpenFGA CLI Commands Reference

This document contains all OpenFGA CLI commands used in this project, including examples and use cases for managing authorization models, stores, and testing.

## Table of Contents

- [Installation](#installation)
- [Store Management](#store-management)
- [Authorization Model Management](#authorization-model-management)
- [Model Transformation](#model-transformation)
- [Relationship Tuples](#relationship-tuples)
- [Testing and Validation](#testing-and-validation)
- [Configuration](#configuration)
- [Troubleshooting](#troubleshooting)

## Installation

### Download and Install OpenFGA CLI

```bash
# Download the latest release for Linux
curl -L -o fga.tar.gz "https://github.com/openfga/cli/releases/download/v0.7.4/fga_0.7.4_linux_amd64.tar.gz"

# Extract and make executable
tar -xzf fga.tar.gz
chmod +x fga

# Move to system PATH (optional)
sudo mv fga /usr/local/bin/

# Verify installation
./fga --version
```

### Alternative: Install via Package Manager

```bash
# Using Go (if available)
go install github.com/openfga/cli/cmd/fga@latest

# Using Homebrew (macOS)
brew install openfga/tap/fga
```

## Store Management

### Create a New Store

```bash
# Create a store with a name
./fga store create --name "my-application-store"

# Example output:
# {
#   "store": {
#     "created_at": "2025-09-18T08:49:24.93938Z",
#     "id": "01K5E077EAVXSMWEM6N2XTVFG7",
#     "name": "my-application-store",
#     "updated_at": "2025-09-18T08:49:24.93938Z"
#   }
# }
```

### List All Stores

```bash
# List all stores
./fga store list

# With pagination
./fga store list --max-pages 5 --page-size 20
```

### Get Store Details

```bash
# Get specific store information
./fga store get --store-id 01K5E077EAVXSMWEM6N2XTVFG7
```

### Delete a Store

```bash
# Delete a store (use with caution!)
./fga store delete --store-id 01K5E077EAVXSMWEM6N2XTVFG7
```

## Authorization Model Management

### Write Authorization Model from DSL

```bash
# Create authorization model from .fga file
./fga model write --store-id 01K5E077EAVXSMWEM6N2XTVFG7 --file etc/auth-model-example.fga

# Example output:
# {
#   "authorization_model_id": "01K5E07J98KVTS1EV0ESQDETHZ"
# }
```

### Write Authorization Model from JSON

```bash
# Create authorization model from JSON file
./fga model write --store-id 01K5E077EAVXSMWEM6N2XTVFG7 --file etc/auth-model-official.json

# With specific format (optional)
./fga model write --store-id 01K5E077EAVXSMWEM6N2XTVFG7 --file auth-model.json --input-format json
```

### Read Authorization Model

```bash
# Get the latest authorization model
./fga model get --store-id 01K5E077EAVXSMWEM6N2XTVFG7

# Get specific model version
./fga model get --store-id 01K5E077EAVXSMWEM6N2XTVFG7 --authorization-model-id 01K5E07J98KVTS1EV0ESQDETHZ
```

### List Authorization Models

```bash
# List all models for a store
./fga model list --store-id 01K5E077EAVXSMWEM6N2XTVFG7

# With pagination
./fga model list --store-id 01K5E077EAVXSMWEM6N2XTVFG7 --max-pages 3 --page-size 10
```

## Model Transformation

### Convert DSL to JSON

```bash
# Transform .fga file to JSON format
./fga model transform --input-format fga --output-format json --file etc/auth-model-example.fga

# Save output to file
./fga model transform --input-format fga --output-format json --file etc/auth-model-example.fga > etc/auth-model-converted.json
```

### Convert JSON to DSL

```bash
# Transform JSON file to .fga format
./fga model transform --input-format json --output-format fga --file etc/auth-model-official.json

# Save output to file
./fga model transform --input-format json --output-format fga --file etc/auth-model-official.json > converted-model.fga
```

### Validate Model Format

```bash
# Validate a DSL model file
./fga model validate --file etc/auth-model-example.fga

# Validate a JSON model file
./fga model validate --file etc/auth-model-official.json --input-format json
```

## Relationship Tuples

### Write Relationship Tuples

```bash
# Write tuples from a file
./fga tuple write --store-id 01K5E077EAVXSMWEM6N2XTVFG7 --file etc/relationship_tuples.yaml

# Write a single tuple
./fga tuple write --store-id 01K5E077EAVXSMWEM6N2XTVFG7 \
  --user "user:alice" \
  --relation "member" \
  --object "group:engineering"
```

### Read Relationship Tuples

```bash
# Read all tuples
./fga tuple read --store-id 01K5E077EAVXSMWEM6N2XTVFG7

# Read tuples for specific object
./fga tuple read --store-id 01K5E077EAVXSMWEM6N2XTVFG7 --object "group:engineering"

# Read tuples for specific user
./fga tuple read --store-id 01K5E077EAVXSMWEM6N2XTVFG7 --user "user:alice"
```

### Delete Relationship Tuples

```bash
# Delete specific tuple
./fga tuple delete --store-id 01K5E077EAVXSMWEM6N2XTVFG7 \
  --user "user:alice" \
  --relation "member" \
  --object "group:engineering"
```

## Testing and Validation

### Check Authorization

```bash
# Check if user has permission
./fga query check --store-id 01K5E077EAVXSMWEM6N2XTVFG7 \
  --user "user:alice" \
  --relation "viewer" \
  --object "resource:document1"

# With specific authorization model
./fga query check --store-id 01K5E077EAVXSMWEM6N2XTVFG7 \
  --authorization-model-id 01K5E07J98KVTS1EV0ESQDETHZ \
  --user "user:alice" \
  --relation "viewer" \
  --object "resource:document1"
```

### List Objects

```bash
# List objects user has access to
./fga query list-objects --store-id 01K5E077EAVXSMWEM6N2XTVFG7 \
  --user "user:alice" \
  --relation "viewer" \
  --type "resource"
```

### List Users

```bash
# List users with access to an object
./fga query list-users --store-id 01K5E077EAVXSMWEM6N2XTVFG7 \
  --object "resource:document1" \
  --relation "viewer" \
  --user-filters "type:user"
```

### Expand Relations

```bash
# Expand a relation to see all users
./fga query expand --store-id 01K5E077EAVXSMWEM6N2XTVFG7 \
  --object "resource:document1" \
  --relation "viewer"
```

## Configuration

### Set Default Store

```bash
# Configure default store ID to avoid repeating --store-id
./fga configure --store-id 01K5E077EAVXSMWEM6N2XTVFG7

# Set API endpoint (if using custom server)
./fga configure --api-url http://localhost:8081

# Set authorization model ID
./fga configure --authorization-model-id 01K5E07J98KVTS1EV0ESQDETHZ
```

### View Current Configuration

```bash
# Show current configuration
./fga configure --show
```

### Environment Variables

```bash
# Alternative to CLI configuration
export FGA_SERVER_URL=http://localhost:8081
export FGA_STORE_ID=01K5E077EAVXSMWEM6N2XTVFG7
export FGA_MODEL_ID=01K5E07J98KVTS1EV0ESQDETHZ

# Use environment variables
./fga query check --user "user:alice" --relation "viewer" --object "resource:doc1"
```

## Troubleshooting

### Debug Mode

```bash
# Enable debug output
./fga --debug model write --store-id 01K5E077EAVXSMWEM6N2XTVFG7 --file auth-model.fga

# Verbose output
./fga --verbose query check --user "user:alice" --relation "viewer" --object "resource:doc1"
```

### Common Error Solutions

#### Connection Issues

```bash
# Check if server is running
curl -s http://localhost:8081/healthz

# Test with specific server URL
./fga --api-url http://localhost:8081 store list
```

#### Model Validation Errors

```bash
# Validate model before writing
./fga model validate --file auth-model.fga

# Check model syntax
./fga model transform --input-format fga --output-format json --file auth-model.fga
```

#### Store Not Found

```bash
# List all stores to find correct ID
./fga store list

# Create new store if needed
./fga store create --name "new-store"
```

## Example Workflows

### Complete Setup Workflow

```bash
#!/bin/bash
# Complete setup for new OpenFGA instance

# 1. Create store
STORE_RESPONSE=$(./fga store create --name "production-store")
STORE_ID=$(echo $STORE_RESPONSE | jq -r '.store.id')
echo "Created store: $STORE_ID"

# 2. Write authorization model
MODEL_RESPONSE=$(./fga model write --store-id $STORE_ID --file auth-model.fga)
MODEL_ID=$(echo $MODEL_RESPONSE | jq -r '.authorization_model_id')
echo "Created model: $MODEL_ID"

# 3. Configure defaults
./fga configure --store-id $STORE_ID --authorization-model-id $MODEL_ID

# 4. Load initial data
./fga tuple write --file initial-tuples.yaml

echo "Setup complete!"
```

### Testing Workflow

```bash
#!/bin/bash
# Test authorization model

STORE_ID="01K5E077EAVXSMWEM6N2XTVFG7"

# Test various scenarios
echo "Testing user permissions..."

# Test 1: Check admin access
./fga query check --store-id $STORE_ID \
  --user "user:admin" \
  --relation "admin" \
  --object "resource:sensitive-doc"

# Test 2: Check viewer access
./fga query check --store-id $STORE_ID \
  --user "user:alice" \
  --relation "viewer" \
  --object "resource:public-doc"

# Test 3: List accessible resources
./fga query list-objects --store-id $STORE_ID \
  --user "user:alice" \
  --relation "viewer" \
  --type "resource"
```

### Model Development Cycle

```bash
#!/bin/bash
# Development cycle for authorization models

# 1. Edit model in DSL format (easier to read/write)
vim auth-model.fga

# 2. Validate syntax
./fga model validate --file auth-model.fga

# 3. Convert to JSON for API use
./fga model transform --input-format fga --output-format json \
  --file auth-model.fga > auth-model.json

# 4. Test with temporary store
TEMP_STORE=$(./fga store create --name "test-$(date +%s)" | jq -r '.store.id')
./fga model write --store-id $TEMP_STORE --file auth-model.fga

# 5. Run tests
./run-authorization-tests.sh $TEMP_STORE

# 6. Clean up
./fga store delete --store-id $TEMP_STORE
```

## Integration with Rust API

### Validate Conversion Logic

```bash
#!/bin/bash
# Script to validate our Rust API conversion against CLI

STORE_ID="01K5E077EAVXSMWEM6N2XTVFG7"
JSON_FILE="etc/auth-model-official.json"

# 1. Create model via CLI (ground truth)
CLI_RESULT=$(./fga model write --store-id $STORE_ID --file $JSON_FILE)
CLI_MODEL_ID=$(echo $CLI_RESULT | jq -r '.authorization_model_id')

# 2. Create model via our API
API_RESULT=$(curl -s -X POST "http://localhost:5001/api/ofga/model-json/$STORE_ID" \
  -H "Content-Type: application/json" \
  -d @$JSON_FILE)
API_MODEL_ID=$(echo $API_RESULT | jq -r '.authorization_model_id')

# 3. Compare results
if [ "$CLI_MODEL_ID" != "null" ] && [ "$API_MODEL_ID" != "null" ]; then
    echo "✅ Both CLI and API succeeded"
    echo "CLI Model ID: $CLI_MODEL_ID"
    echo "API Model ID: $API_MODEL_ID"
else
    echo "❌ Conversion mismatch detected"
    echo "CLI Result: $CLI_RESULT"
    echo "API Result: $API_RESULT"
fi
```

## References

- [OpenFGA CLI Documentation](https://openfga.dev/docs/getting-started/setup-cli)
- [OpenFGA DSL Documentation](https://openfga.dev/docs/modeling/getting-started)
- [OpenFGA API Documentation](https://openfga.dev/api)
- [GitHub Repository](https://github.com/openfga/cli)

## Version Information

- **OpenFGA CLI Version**: v0.7.4
- **Last Updated**: September 18, 2025
- **Compatible with**: OpenFGA Server v1.x

---

**Note**: Always test commands in a development environment before using in production. Store IDs and model IDs in examples are for illustration only.
