# OpenFGA Rust Client Generator

This project demonstrates how to generate a Rust client for OpenFGA using the official protocol buffer definitions from the [OpenFGA API repository](https://github.com/openfga/api).

## Overview

Instead of creating custom `.proto` files, this implementation uses the standard OpenFGA protocol buffer definitions, ensuring full compatibility with the official OpenFGA API.

## Project Structure

This project is organized as a Cargo workspace with three main crates:

```
openfga-demo/
├── Cargo.toml                    # Workspace configuration
├── client-builder/               # Proto compilation and code generation tool
│   ├── Cargo.toml
│   ├── build.rs                  # Build script for proto compilation
│   └── src/lib.rs               # Build tool (not a client library)
    └── proto/                        # Proto files (13 files total)
        ├── google/api/              # Google API annotations (4 files)
        ├── openfga/v1/              # OpenFGA service definitions (5 files)
        ├── protoc-gen-openapiv2/    # OpenAPI annotations (2 files)
        └── validate/                # Validation definitions (1 file)
├── openfga-client/               # Public-facing client library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs               # High-level client wrapper
│       └── generated.rs         # Generated code (auto-created by client-builder)
├── service-demo/                 # Demo application
│   ├── Cargo.toml
│   ├── src/                     # Demo application code
│   └── examples/                # Usage examples
```

### Crate Responsibilities

- **`client-builder`**: A development tool that compiles `.proto` files into Rust code using `tonic-build` and `prost-build`, then copies the generated code to `openfga-client/src/generated.rs`. External users don't need to depend on this crate.
- **`openfga-client`**: The public-facing library that external users should depend on. Contains both the generated low-level gRPC client and a high-level, user-friendly wrapper. This crate has no build-time dependencies and can be used independently.
- **`service-demo`**: Contains demo application code showing how to use the OpenFGA client in a real application context.

## Setup

### Workspace Configuration

The project uses a Cargo workspace to manage the three modules. The root `Cargo.toml` defines shared dependencies:

```toml
[workspace]
members = ["client-builder", "openfga-client", "service-demo"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"

[workspace.dependencies]
tonic = "0.12"
prost = "0.13"
prost-types = "0.13"
prost-wkt = "0.6"
prost-wkt-types = "0.6"
tokio = { version = "1.35.1", features = ["full"] }
```

## Protocol Buffer Dependencies and Setup

### Why We Need Multiple Repositories

The OpenFGA protocol buffer definitions have dependencies on several external proto files. Here's why each repository is needed and what files we extract:

#### 1. OpenFGA API Repository
**Repository**: [https://github.com/openfga/api](https://github.com/openfga/api)  
**Why needed**: Contains the official OpenFGA protocol buffer definitions  
**Files extracted**:
```
proto/openfga/v1/
├── authzmodel.proto              # Authorization model definitions (TypeDefinition, Userset, etc.)
├── errors_ignore.proto           # Error definitions and codes
├── openapi.proto                 # OpenAPI-specific message definitions
├── openfga.proto                 # Core types (Object, User, TupleKey, Store, etc.)
├── openfga_service_consistency.proto  # Consistency preference definitions
└── openfga_service.proto         # Main service definition with all RPC methods
```
**Relevance**: These are the core OpenFGA types and service definitions. Without these, we can't generate a client that communicates with OpenFGA servers.

#### 2. Google APIs Repository
**Repository**: [https://github.com/googleapis/googleapis](https://github.com/googleapis/googleapis)  
**Why needed**: OpenFGA protos use Google API annotations for HTTP/gRPC mapping and field behavior  
**Files extracted**:
```
proto/google/api/
├── annotations.proto             # HTTP/gRPC service annotations (google.api.http)
├── field_behavior.proto          # Field behavior annotations (REQUIRED, OPTIONAL)
├── http.proto                    # HTTP rule definitions for REST API mapping
└── visibility.proto              # API visibility and deprecation annotations
```
**Relevance**: 
- `annotations.proto`: Enables gRPC services to be exposed as REST APIs
- `field_behavior.proto`: Marks fields as required/optional for validation
- `http.proto`: Defines how gRPC methods map to HTTP endpoints
- `visibility.proto`: Controls API visibility and deprecation warnings

#### 3. Protoc Gen Validate Repository
**Repository**: [https://github.com/bufbuild/protoc-gen-validate](https://github.com/bufbuild/protoc-gen-validate)  
**Why needed**: OpenFGA uses validation annotations to ensure data integrity  
**Files extracted**:
```
proto/validate/
└── validate.proto                # Validation rules (string patterns, number ranges, etc.)
```
**Relevance**: Provides validation annotations like:
- String pattern validation (e.g., `pattern: "^[^:#@\\s]{1,254}$"` for object types)
- Field requirements (`ignore_empty: false`)
- Length constraints and format validation

#### 4. gRPC Gateway Repository
**Repository**: [https://github.com/grpc-ecosystem/grpc-gateway](https://github.com/grpc-ecosystem/grpc-gateway)  
**Why needed**: OpenFGA uses OpenAPI v2 annotations for API documentation generation  
**Files extracted**:
```
proto/protoc-gen-openapiv2/options/
├── annotations.proto             # OpenAPI v2 field and method annotations
└── openapiv2.proto              # OpenAPI v2 schema definitions (Swagger, Operation, etc.)
```
**Relevance**: 
- `annotations.proto`: Provides OpenAPI annotations for fields and methods
- `openapiv2.proto`: Defines OpenAPI v2 schema structures for documentation
- Used for generating API documentation and client SDKs

### Dependency Chain

The dependency chain looks like this:
```
openfga_service.proto
├── imports google/api/annotations.proto (for HTTP mapping)
├── imports google/api/field_behavior.proto (for field validation)
├── imports validate/validate.proto (for data validation)
├── imports protoc-gen-openapiv2/options/annotations.proto (for OpenAPI docs)
├── imports openfga.proto
│   ├── imports google/api/field_behavior.proto
│   ├── imports validate/validate.proto
│   └── imports protoc-gen-openapiv2/options/annotations.proto
├── imports authzmodel.proto
│   ├── imports google/api/field_behavior.proto
│   ├── imports validate/validate.proto
│   └── imports protoc-gen-openapiv2/options/annotations.proto
└── imports errors_ignore.proto
    └── imports protoc-gen-openapiv2/options/annotations.proto
```

### Setup Process

To set up the clean proto directory, we:

1. **Clone the required repositories**:
   ```bash
   git clone https://github.com/openfga/api.git openfga-api
   git clone https://github.com/googleapis/googleapis.git
   git clone https://github.com/bufbuild/protoc-gen-validate.git
   git clone https://github.com/grpc-ecosystem/grpc-gateway.git
   ```

2. **Create the clean proto directory structure**:
   ```bash
   mkdir -p proto/{openfga/v1,google/api,validate,protoc-gen-openapiv2/options}
   ```

3. **Copy only the required files**:
   ```bash
   # OpenFGA core files
   cp openfga-api/openfga/v1/*.proto proto/openfga/v1/
   
   # Google API annotations
   cp googleapis/google/api/{annotations.proto,field_behavior.proto,http.proto,visibility.proto} proto/google/api/
   
   # Validation rules
   cp protoc-gen-validate/validate/validate.proto proto/validate/
   
   # OpenAPI v2 annotations
   cp grpc-gateway/protoc-gen-openapiv2/options/{annotations.proto,openapiv2.proto} proto/protoc-gen-openapiv2/options/
   ```

4. **Clean up the large repositories**:
   ```bash
   rm -rf openfga-api googleapis protoc-gen-validate grpc-gateway
   ```

### Final Proto Directory Structure

```
proto/
├── google/api/                    # Google API annotations (4 files)
│   ├── annotations.proto          # gRPC to HTTP mapping
│   ├── field_behavior.proto       # Field validation behavior
│   ├── http.proto                 # HTTP rule definitions
│   └── visibility.proto           # API visibility control
├── openfga/v1/                   # Official OpenFGA definitions (6 files)
│   ├── authzmodel.proto           # Authorization model types
│   ├── errors_ignore.proto        # Error definitions
│   ├── openapi.proto              # OpenAPI-specific messages
│   ├── openfga.proto              # Core OpenFGA types
│   ├── openfga_service_consistency.proto  # Consistency preferences
│   └── openfga_service.proto      # Main service definition
├── protoc-gen-openapiv2/options/ # OpenAPI v2 annotations (2 files)
│   ├── annotations.proto          # OpenAPI field annotations
│   └── openapiv2.proto           # OpenAPI v2 schema definitions
└── validate/                     # Validation rules (1 file)
    └── validate.proto             # Data validation annotations
```

**Total**: 13 essential files instead of 4 large repositories (~500MB+ → ~2MB)

## Build Script Documentation

The `client-builder/build.rs` file is a Cargo build script that runs before the main compilation. It's responsible for converting protocol buffer files into Rust code. Here's a detailed breakdown:

### Complete Build Script

```rust
use prost_wkt_build::{FileDescriptorSet, Message};
use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "proto";
    
    // Tell cargo to rerun this build script if the proto files change
    println!("cargo:rerun-if-changed={}", proto_root);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let descriptor_path = out_dir.join("descriptors.bin");

    // Configure tonic-build with proper Google API types support
    tonic_build::configure()
        .build_server(false) // We only need the client
        .build_client(true)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        // Map Google well-known types to prost-wkt-types
        .extern_path(".google.protobuf.Timestamp", "::prost_wkt_types::Timestamp")
        .extern_path(".google.protobuf.Struct", "::prost_wkt_types::Struct")
        .extern_path(".google.protobuf.Value", "::prost_wkt_types::Value")
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(&["proto/openfga/v1/openfga_service.proto"], &["proto"])?;

    // Handle well-known types with serde support
    let descriptor_bytes = std::fs::read(descriptor_path)?;
    let descriptor = FileDescriptorSet::decode(&descriptor_bytes[..])?;
    prost_wkt_build::add_serde(out_dir, descriptor);

    Ok(())
}
```

### Step-by-Step Explanation

#### 1. Dependencies and Setup
```rust
use prost_wkt_build::{FileDescriptorSet, Message};
use std::{env, path::PathBuf};
```

**Purpose**: Import required dependencies for protocol buffer compilation and file system operations.
- `prost_wkt_build`: Handles Google well-known types with serde support
- `std::{env, path::PathBuf}`: For environment variables and file path manipulation

#### 2. Proto Root Configuration

```rust
let proto_root = "proto";
```

**Purpose**: Define the root directory containing our protocol buffer files.
**Why**: Centralized configuration makes it easy to change the proto directory location if needed.


#### 3. Build Cache Invalidation

```rust
println!("cargo:rerun-if-changed={}", proto_root);
```

**Purpose**: Tell Cargo to rerun this build script whenever any file in the `proto` directory changes.
**Why**: Ensures that changes to protocol buffer files trigger regeneration of Rust code.
**How it works**: Cargo monitors the specified path and reruns the build script when files are modified.

#### 4. Output Directory Setup

```rust
let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
let descriptor_path = out_dir.join("descriptors.bin");
```

**Purpose**: Set up paths for generated files.

- `OUT_DIR`: Cargo's designated directory for build script output
- `descriptors.bin`: File descriptor set for prost-wkt processing

**Why**: Generated files must go in `OUT_DIR` to be included in the final binary.

#### 5. Tonic Build Configuration
```rust
tonic_build::configure()
    .build_server(false) // We only need the client
    .build_client(true)
```
**Purpose**: Configure what code to generate.
- `build_server(false)`: Don't generate server-side code (we're building a client)
- `build_client(true)`: Generate client-side gRPC code

**Why**: Reduces generated code size and compilation time by only generating what we need.

#### 6. Serde Integration
```rust
.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
.type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
```
**Purpose**: Add serde serialization support to all generated types.
- First line: Adds `#[derive(serde::Serialize, serde::Deserialize)]` to all structs/enums
- Second line: Converts field names to camelCase in JSON (e.g., `store_id` → `storeId`)

**Why**: Enables easy JSON serialization/deserialization for web APIs and configuration files.

#### 7. Google Well-Known Types Mapping
```rust
.extern_path(".google.protobuf.Timestamp", "::prost_wkt_types::Timestamp")
.extern_path(".google.protobuf.Struct", "::prost_wkt_types::Struct")
.extern_path(".google.protobuf.Value", "::prost_wkt_types::Value")
```
**Purpose**: Map Google's well-known types to Rust types that support serde.
**Why**: 
- Default prost types don't support serde serialization
- `prost_wkt_types` provides serde-compatible versions
- Ensures consistent JSON representation of timestamps and dynamic values

**Types mapped**:
- `Timestamp`: For created_at, updated_at fields
- `Struct`: For dynamic context objects
- `Value`: For arbitrary JSON values

#### 8. File Descriptor Set
```rust
.file_descriptor_set_path(&descriptor_path)
```
**Purpose**: Generate a file descriptor set for post-processing.
**Why**: Required by `prost_wkt_build` to add serde support to well-known types.

#### 9. Protocol Buffer Compilation
```rust
.compile_protos(&["proto/openfga/v1/openfga_service.proto"], &["proto"])?;
```
**Purpose**: Compile the protocol buffer files into Rust code.
**Parameters**:
- First array: Entry point proto files to compile
- Second array: Include directories for proto imports

**Why only `openfga_service.proto`**: This file imports all other required proto files, so compiling it automatically includes all dependencies.

#### 10. Well-Known Types Post-Processing
```rust
let descriptor_bytes = std::fs::read(descriptor_path)?;
let descriptor = FileDescriptorSet::decode(&descriptor_bytes[..])?;
prost_wkt_build::add_serde(out_dir, descriptor);
```
**Purpose**: Add serde support to Google well-known types in the generated code.
**Process**:
1. Read the file descriptor set generated by tonic-build
2. Decode it into a structured format
3. Use `prost_wkt_build` to add serde implementations

**Why**: Ensures that `Timestamp`, `Struct`, and `Value` types can be serialized to/from JSON.

### Generated Output

The build script generates several files in the `OUT_DIR`:

1. **`openfga.v1.rs`**: Main generated module containing:
   - All message types (structs)
   - All enum types
   - Service client code
   - Serde implementations

2. **`descriptors.bin`**: Binary file descriptor set (intermediate file)

3. **Additional modules**: For each imported package (e.g., `google.api.rs`, `validate.rs`)

### Build Process Flow

```
build.rs runs
    ↓
Reads proto files from proto/ directory
    ↓
Compiles proto files using protoc + tonic-build
    ↓
Generates Rust code in OUT_DIR
    ↓
Adds serde support to well-known types
    ↓
Generated code is available for use via tonic::include_proto!()
```

### Error Handling

The build script uses `?` operator for error propagation. Common errors:
- **Proto file not found**: Check proto directory structure
- **Import resolution failure**: Ensure all required proto files are present
- **Compilation errors**: Usually due to missing dependencies or syntax errors in proto files

### Performance Considerations

- **Incremental builds**: Only reruns when proto files change
- **Minimal compilation**: Only compiles the main service proto (dependencies are automatically included)
- **Optimized output**: Generates only client code, not server code

## Usage

### Generating the Client Code

The client code generation is a two-step process:

1. **Generate the code** (run this when proto files change):
```bash
cargo build -p client-builder
```
This compiles the `.proto` files and copies the generated code to `openfga-client/src/generated.rs`.

2. **Build the client library**:
```bash
cargo build -p openfga-client
```

### For External Users

If you're using this as a library, you only need to depend on `openfga-client`:

```toml
[dependencies]
openfga-client = { path = "path/to/openfga-client" }
# or from a git repository:
# openfga-client = { git = "https://github.com/your-repo/openfga-demo", package = "openfga-client" }
```

### Building the Project

Build all modules in the workspace:

```bash
# Build everything
cargo build

# Build specific modules
cargo build -p client-builder  # Generates proto code and copies to openfga-client
cargo build -p openfga-client   # Public-facing client library
cargo build -p demo             # Demo service
```

### Using the OpenFGA Client

The high-level client is available in the `openfga-client` crate:

```rust
use openfga_client::OpenFGAClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new OpenFGA client
    let mut client = OpenFGAClient::new("http://localhost:8080".to_string()).await?;
    
    // Use the client...
    Ok(())
}
```

### Writing Relationships

```rust
let write_request = OpenFGAClient::create_write_request(
    store_id.clone(),
    "document".to_string(),
    "doc1".to_string(),
    "reader".to_string(),
    "user".to_string(),
    "alice".to_string(),
);

match client.write(write_request).await {
    Ok(_) => println!("✓ Successfully wrote relationship"),
    Err(e) => println!("✗ Failed to write relationship: {}", e),
}
```

### Checking Permissions

```rust
let check_request = OpenFGAClient::create_check_request(
    store_id.clone(),
    "document:doc1".to_string(),
    "reader".to_string(),
    "user:alice".to_string(),
);

match client.check(check_request).await {
    Ok(response) => {
        let check_response = response.into_inner();
        if check_response.allowed {
            println!("✓ Access granted");
        } else {
            println!("✗ Access denied");
        }
    }
    Err(e) => println!("✗ Failed to check access: {}", e),
}
```

### Available Methods

The `OpenFGAClient` provides wrapper methods for all OpenFGA API endpoints:

- `read()` - Read tuples from the store
- `write()` - Write tuples to the store  
- `check()` - Check if a user has a relation to an object
- `expand()` - Expand a userset
- `read_authorization_model()` - Get authorization model
- `write_authorization_model()` - Write authorization model
- `read_authorization_models()` - List authorization models
- `get_store()` - Get store information
- `list_stores()` - List stores
- `create_store()` - Create a new store
- `delete_store()` - Delete a store
- `list_objects()` - List objects
- `read_changes()` - Read changes stream

### Direct gRPC Client Access

For advanced usage, you can access the underlying gRPC client:

```rust
let grpc_client = client.inner();
// Use grpc_client directly for custom requests
```

## Generated Types

The build process generates all the standard OpenFGA types including:

- `CheckRequest`, `CheckResponse`
- `WriteRequest`, `WriteResponse`  
- `ReadRequest`, `ReadResponse`
- `AuthorizationModel`, `TypeDefinition`
- `TupleKey`, `Tuple`, `TupleChange`
- `Store`, `Object`, `User`
- And many more...

All generated types include serde serialization support and follow the camelCase naming convention for JSON compatibility.

## Examples

### Running the Example

See `service-demo/examples/openfga_client_example.rs` for a complete working example.

To run the example:
```bash
# From the workspace root
cargo run -p demo --example openfga_client_example

# Or from the service-demo directory
cd service-demo
cargo run --example openfga_client_example
```

## Benefits of This Approach

1. **Official Compatibility** - Uses the exact same protocol buffer definitions as the official OpenFGA server
2. **Minimal Dependencies** - Only includes the essential proto files needed for compilation
3. **Complete API Coverage** - All OpenFGA API methods and types are available
4. **Type Safety** - Full Rust type safety with generated structs and enums
5. **Serde Support** - Built-in JSON serialization/deserialization
6. **Clean Structure** - Organized proto directory without unnecessary files
7. **Fast Builds** - No need to download large repositories during build
8. **Self-Contained** - All required proto files are included in the project

This approach ensures that your Rust client is always compatible with the official OpenFGA API specification while maintaining a clean and minimal project structure.

## Troubleshooting

### Common Issues and Solutions

#### 1. Proto File Not Found Errors
**Error**: `protoc failed: some/file.proto: File not found`
**Cause**: Missing protocol buffer files in the proto directory
**Solution**: 
```bash
# Verify all required files are present
find client-builder/proto -name "*.proto" | wc -l  # Should be 13 files

# Re-copy missing files from source repositories if needed
```

#### 2. Import Resolution Failures
**Error**: `Import "google/api/annotations.proto" was not found`
**Cause**: Incorrect proto directory structure or missing import files
**Solution**:
```bash
# Check directory structure matches expected layout
tree client-builder/proto/
# Ensure all Google API files are present
ls client-builder/proto/google/api/
```

#### 3. Type Not Defined Errors
**Error**: `"SomeType" is not defined`
**Cause**: Missing dependency proto files or incorrect import order
**Solution**: Ensure all OpenFGA proto files are present and verify dependency chain

#### 4. Serde Compilation Errors
**Error**: Serde trait not implemented for certain types
**Cause**: Issues with well-known types mapping or prost-wkt-build configuration
**Solution**:
```rust
// Verify these mappings are present in build.rs
.extern_path(".google.protobuf.Timestamp", "::prost_wkt_types::Timestamp")
.extern_path(".google.protobuf.Struct", "::prost_wkt_types::Struct")
.extern_path(".google.protobuf.Value", "::prost_wkt_types::Value")
```

#### 5. Build Script Compilation Errors
**Error**: Build script fails to compile
**Cause**: Missing build dependencies
**Solution**:
```toml
# Ensure these are in Cargo.toml [build-dependencies]
tonic-build = "0.12"
prost-build = "0.13"
prost-wkt-build = "0.6"
```

### Updating Proto Files

To update to newer versions of OpenFGA:

1. **Backup current proto directory**:
   ```bash
   cp -r client-builder/proto client-builder/proto.backup
   ```

2. **Clone latest repositories**:
   ```bash
   git clone https://github.com/openfga/api.git openfga-api-new
   # Clone other repos as needed
   ```

3. **Copy updated files**:
   ```bash
   cp openfga-api-new/openfga/v1/*.proto client-builder/proto/openfga/v1/
   ```

4. **Test the build**:
   ```bash
   cargo clean && cargo build
   ```

5. **Clean up**:
   ```bash
   rm -rf openfga-api-new client-builder/proto.backup
   ```

### Debugging Generated Code

To examine the generated Rust code:

```bash
# Find the generated files
find target -name "*.rs" -path "*/out/*" | grep openfga

# View the generated code
cat target/debug/build/*/out/openfga.v1.rs | head -50
```

### Performance Optimization

For faster builds:
- Use `cargo check` during development instead of `cargo build`
- Consider using `sccache` for build caching
- Use incremental compilation: `CARGO_INCREMENTAL=1`



## References

- [OpenFGA API](https://github.com/openfga/api)
- [OpenFGA Rust Client](https://github.com/liamwh/openfga-rs)

