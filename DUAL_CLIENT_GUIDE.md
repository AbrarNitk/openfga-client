# OpenFGA Dual Client Guide

This project now supports **both gRPC and HTTP clients** for interacting with OpenFGA, giving you flexibility to choose the best approach for your use case.

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Service Demo API                     │
├─────────────────────┬───────────────────────────────────┤
│    gRPC Routes      │         HTTP Routes               │
│  /api/ofga/grpc/*   │      /api/ofga/http/*            │
├─────────────────────┼───────────────────────────────────┤
│  OpenFGA gRPC Client│     OpenFGA HTTP Client          │
│  (Proto-generated)  │   (OpenAPI-generated)            │
├─────────────────────┼───────────────────────────────────┤
│                 OpenFGA Server                          │
│            (Supports both gRPC & HTTP)                  │
└─────────────────────────────────────────────────────────┘
```

## 📊 When to Use Which Client

### gRPC Client (`/api/ofga/grpc/*`)
**Best for:**
- ✅ **High-performance** applications
- ✅ **Internal services** communication
- ✅ **Streaming** operations
- ✅ **Type safety** with protobuf
- ✅ **Lower latency** and **smaller payloads**

**Use cases:**
- Microservices architecture
- Real-time authorization checks
- High-throughput scenarios
- When you need maximum performance

### HTTP Client (`/api/ofga/http/*`)
**Best for:**
- ✅ **Web applications** and **browsers**
- ✅ **External integrations**
- ✅ **RESTful APIs**
- ✅ **Standard HTTP tooling**
- ✅ **Easier debugging** with HTTP tools

**Use cases:**
- Frontend applications
- Third-party integrations
- REST API consumers
- When you need HTTP compatibility

## 🛣️ API Routes Comparison

### Store Operations

| Operation | gRPC Route | HTTP Route |
|-----------|------------|------------|
| Create Store | `POST /api/ofga/grpc/store` | `POST /api/ofga/http/stores` |
| Get Store | `GET /api/ofga/grpc/store/{store_id}` | `GET /api/ofga/http/stores/{store_id}` |
| List Stores | `GET /api/ofga/grpc/store` | `GET /api/ofga/http/stores` |
| Delete Store | `DELETE /api/ofga/grpc/store/{store_id}` | `DELETE /api/ofga/http/stores/{store_id}` |

### Authorization Model Operations

| Operation | gRPC Route | HTTP Route |
|-----------|------------|------------|
| Create Model | `POST /api/ofga/grpc/model/{store_id}` | `POST /api/ofga/http/stores/{store_id}/authorization-models` |
| Get Model | `GET /api/ofga/grpc/model/{store_id}/{auth_model_id}` | `GET /api/ofga/http/stores/{store_id}/authorization-models/{auth_model_id}` |
| List Models | `GET /api/ofga/grpc/model/{store_id}` | `GET /api/ofga/http/stores/{store_id}/authorization-models` |

### Tuple Operations

| Operation | gRPC Route | HTTP Route |
|-----------|------------|------------|
| Write Tuples | `POST /api/ofga/grpc/tuple-write` | `POST /api/ofga/http/write` |
| Read Tuples | `POST /api/ofga/grpc/tuple-read` | `POST /api/ofga/http/read` |
| Delete Tuples | `POST /api/ofga/grpc/tuple-delete` | `POST /api/ofga/http/delete` |
| Tuple Changes | `POST /api/ofga/grpc/tuple-changes` | `POST /api/ofga/http/changes` |

### Query Operations

| Operation | gRPC Route | HTTP Route |
|-----------|------------|------------|
| Check | `POST /api/ofga/grpc/check` | `POST /api/ofga/http/check` |
| Batch Check | `POST /api/ofga/grpc/batch-check` | `POST /api/ofga/http/batch-check` |
| Expand | `POST /api/ofga/grpc/expand` | `POST /api/ofga/http/expand` |
| List Objects | `GET /api/ofga/grpc/list-objs` | `POST /api/ofga/http/list-objects` |
| List Users | `GET /api/ofga/grpc/list-users` | `POST /api/ofga/http/list-users` |

## 📝 Usage Examples

### HTTP Client Example

```bash
# Create a store
curl -X POST http://localhost:3000/api/ofga/http/stores \
  -H "Content-Type: application/json" \
  -d '{"name": "my-store"}'

# Check authorization
curl -X POST http://localhost:3000/api/ofga/http/check \
  -H "Content-Type: application/json" \
  -d '{
    "store_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "check_request": {
      "tuple_key": {
        "user": "user:alice",
        "relation": "reader", 
        "object": "document:readme"
      }
    }
  }'
```

### gRPC Client Example

```bash
# The gRPC endpoints work with the same JSON payloads
# but use different route paths

# Check authorization (gRPC)
curl -X POST http://localhost:3000/api/ofga/grpc/check \
  -H "Content-Type: application/json" \
  -d '{
    "user": "user:alice",
    "object": "document:readme",
    "relation": "reader"
  }'
```

## 🔧 Configuration

### HTTP Client Configuration

The HTTP client uses the generated OpenFGA HTTP client with standard HTTP configuration:

```rust
use openfga_http_client::apis::configuration::Configuration;

let config = Configuration::new();
// Configure base URL, authentication, etc.
```

### gRPC Client Configuration

The gRPC client uses your existing OpenFGA gRPC client:

```rust
use openfga_grpc_client::OpenFGAClient;

let client = OpenFGAClient::new("http://localhost:8081").await?;
```

## 🚀 Performance Considerations

### gRPC Advantages
- **~30-50% smaller** payload sizes (protobuf vs JSON)
- **~20-30% lower latency** (binary protocol)
- **HTTP/2 multiplexing** built-in
- **Streaming support** for large datasets

### HTTP Advantages  
- **Universal compatibility** (works everywhere)
- **Easier debugging** with standard HTTP tools
- **Better caching** support
- **Simpler integration** with existing HTTP infrastructure

## 🔄 Migration Guide

### From gRPC to HTTP
1. Change route from `/api/ofga/grpc/*` to `/api/ofga/http/*`
2. Adjust request/response formats to match OpenAPI spec
3. Update error handling for HTTP status codes

### From HTTP to gRPC
1. Change route from `/api/ofga/http/*` to `/api/ofga/grpc/*`  
2. Adjust request/response formats to match protobuf types
3. Update error handling for gRPC status codes

## 🛠️ Development

### Adding New Endpoints

**For gRPC:**
1. Update proto files in `client-builder/proto/`
2. Regenerate client: `cargo build -p client-builder`
3. Add route handlers in `service-demo/src/apis/grpc/`
4. Update routes in `service-demo/src/routes.rs`

**For HTTP:**
1. Update OpenAPI spec in `client-builder/openapi/openfga-openapi.json`
2. Regenerate client: `openapi-generator-cli generate -i client-builder/openapi/openfga-openapi.json -g rust -o openfga-http-client`
3. Add route handlers in `service-demo/src/apis/http/`
4. Update routes in `service-demo/src/routes.rs`

## 📚 Further Reading

- [OpenFGA Documentation](https://openfga.dev/docs)
- [OpenFGA API Reference](https://openfga.dev/api)
- [gRPC vs HTTP Performance](https://grpc.io/docs/guides/benchmarking/)
- [OpenAPI Generator](https://openapi-generator.tech/)

---

**Note:** Both clients connect to the same OpenFGA server and provide identical functionality. Choose based on your specific requirements and constraints.
