# OpenFGA Rust HTTP Client Generation

This document explains how to generate the Rust HTTP client from the OpenAPI JSON specification without requiring Maven.

## Quick Start

### Method 1: Using Docker (Recommended - No Java/Maven required)

```bash
# Generate using Docker (most reliable, no local dependencies)
./scripts/generate-http-client-docker.sh
```

**One-liner command:**
```bash
docker run --rm -v "${PWD}:/local" openapitools/openapi-generator-cli:v7.2.0 generate -i "/local/client-builder/openapi/openfga-openapi.json" -g rust -o "/local/openfga-http-client-new" --additional-properties=packageName=openfga_http_client,packageVersion=0.1.0,library=reqwest,supportAsync=true,useSingleRequestParameter=false --package-name openfga_http_client
```

### Method 2: Using npm/npx (Requires Node.js)

```bash
# Generate using npm (if you have Node.js installed)
./scripts/generate-http-client-npm.sh
```

**One-liner command:**
```bash
npx @openapitools/openapi-generator-cli generate -i "client-builder/openapi/openfga-openapi.json" -g rust -o "openfga-http-client-new" --additional-properties=packageName=openfga_http_client,packageVersion=0.1.0,library=reqwest,supportAsync=true,useSingleRequestParameter=false --package-name openfga_http_client
```

### Method 3: Direct JAR Download (Requires Java)

```bash
# Generate by downloading JAR directly (no Maven required)
./scripts/generate-http-client-direct.sh
```

### Method 4: Auto-detect (Tries all methods)

```bash
# Automatically selects the best available method
./scripts/generate-http-client.sh
```

## Prerequisites

Before generating the client, ensure you have the OpenAPI JSON specification:

```bash
# Generate the OpenAPI spec from protobuf files
cd client-builder
cargo build
```

This will create `client-builder/openapi/openfga-openapi.json` which is used as input for client generation.

## Generated Client Structure

The generated client will be created in the `openfga-http-client/` directory with the following structure:

```
openfga-http-client/
├── Cargo.toml
├── README.md
├── docs/
│   └── *.md (API documentation)
├── src/
│   ├── lib.rs
│   ├── apis/
│   │   ├── mod.rs
│   │   ├── configuration.rs
│   │   ├── stores_api.rs
│   │   ├── authorization_models_api.rs
│   │   ├── relationship_tuples_api.rs
│   │   └── relationship_queries_api.rs
│   └── models/
│       ├── mod.rs
│       └── *.rs (data models)
└── .openapi-generator/
    └── FILES
```

## Configuration Options

The generator uses the following configuration:

- **Generator**: `rust` (Rust language)
- **Library**: `reqwest` (HTTP client)
- **Package Name**: `openfga_http_client`
- **Package Version**: `0.1.0`
- **Async Support**: `true`
- **Single Request Parameter**: `false`

## Troubleshooting

### Docker Issues
- Make sure Docker is running
- Check Docker permissions if on Linux

### npm/npx Issues
- Update Node.js to the latest LTS version
- Clear npm cache: `npm cache clean --force`

### Java Issues
- Ensure Java 11+ is installed
- Check `JAVA_HOME` environment variable

### OpenAPI Spec Not Found
- Run `cargo build` in `client-builder/` first
- Check that `client-builder/openapi/openfga-openapi.json` exists

## Manual Generation

If the scripts don't work, you can generate manually:

### Using Docker:
```bash
docker run --rm \
  -v "${PWD}:/local" \
  openapitools/openapi-generator-cli:v7.2.0 generate \
  -i "/local/client-builder/openapi/openfga-openapi.json" \
  -g rust \
  -o "/local/openfga-http-client-manual" \
  --additional-properties=packageName=openfga_http_client \
  --additional-properties=packageVersion=0.1.0 \
  --additional-properties=library=reqwest \
  --additional-properties=supportAsync=true \
  --additional-properties=useSingleRequestParameter=false \
  --package-name openfga_http_client
```

### Using Downloaded JAR:
```bash
# Download the JAR
curl -L -o openapi-generator-cli.jar https://repo1.maven.org/maven2/org/openapitools/openapi-generator-cli/7.2.0/openapi-generator-cli-7.2.0.jar

# Generate the client
java -jar openapi-generator-cli.jar generate \
  -i "client-builder/openapi/openfga-openapi.json" \
  -g rust \
  -o "openfga-http-client-manual" \
  --additional-properties=packageName=openfga_http_client \
  --additional-properties=packageVersion=0.1.0 \
  --additional-properties=library=reqwest \
  --additional-properties=supportAsync=true \
  --additional-properties=useSingleRequestParameter=false \
  --package-name openfga_http_client
```

## Integration

After generating the client, add it to your workspace:

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "client-builder",
    "openfga-client", 
    "service-demo",
    "openfga-http-client",  # Add this line
]
```

Then use it in your applications:

```toml
# In your application's Cargo.toml
[dependencies]
openfga-http-client = { path = "../openfga-http-client" }
```
