#!/bin/bash

# Script to start OpenFGA server using Docker

echo "Starting OpenFGA server with Docker..."
echo "======================================"

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "Error: Docker is not installed or not in PATH"
    exit 1
fi

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    echo "Error: Docker is not running"
    exit 1
fi

echo "Starting OpenFGA server..."
echo "Ports:"
echo "  - 8080: HTTP API"
echo "  - 8081: gRPC API"
echo "  - 3000: Playground (web UI)"
echo ""

# Start OpenFGA with Docker
docker run --rm -it \
  -p 8080:8080 \
  -p 8081:8081 \
  -p 3000:3000 \
  openfga/openfga run \
  --playground-enabled

echo ""
echo "OpenFGA server stopped."
