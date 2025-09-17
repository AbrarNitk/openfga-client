# OpenFGA Client Builder

This is a development tool for generating the OpenFGA Rust client code from protocol buffer definitions.

## Purpose

The `client-builder` crate:
1. Compiles `.proto` files from the `proto/` directory 
2. Generates Rust gRPC client code and Protobuf types
3. Copies the generated code to `openfga-client/src/generated.rs`

## Usage

To regenerate the client code (run this when proto files change):

```bash
cargo build -p client-builder
```

This will:
- Compile all the OpenFGA protocol buffer definitions
- Generate optimized Rust code with `serde` support
- Copy the generated code to the `openfga-client` crate

## Important Notes

- **External users don't need this crate** - they should depend on `openfga-client` directly
- This is a build tool, not a library for end users
- The generated code is automatically copied to `openfga-client/src/generated.rs`
- Only needs to be run when protocol buffer definitions change

## Dependencies

This crate requires:
- `tonic-build` and `prost-build` for code generation
- `prost-wkt-build` for well-known types with serde support
- Protocol buffer files in the `proto/` directory

The generated code includes proper `serde` serialization support and maps Google well-known types to `prost-wkt-types` for better compatibility.
