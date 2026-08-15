#!/usr/bin/env bash
set -e

# Detect platform
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
    PLATFORM="windows"
    BINARY_EXT=".exe"
else
    PLATFORM="linux"
    BINARY_EXT=""
fi

echo "Building for $PLATFORM"
echo ""

# Create output directories
mkdir -p "OpenE2E-CLI_${PLATFORM}/bin"
mkdir -p "OpenE2E-GUI_${PLATFORM}/bin"

# Build CLI
echo "Building CLI..."
cargo build --release

# Build GUI with features
echo "Building GUI..."
cargo build --release --features gui

echo "Packaging binaries..."

# Copy binaries to output directories
cp "target/release/OpenE2E${BINARY_EXT}" "OpenE2E-CLI_${PLATFORM}/bin/OpenE2E${BINARY_EXT}"
cp "target/release/OpenE2E${BINARY_EXT}" "OpenE2E-GUI_${PLATFORM}/bin/OpenE2E${BINARY_EXT}"

echo ""
echo "Build succeeded:"
echo "   CLI: ./OpenE2E-CLI_${PLATFORM}/bin/OpenE2E${BINARY_EXT}"
echo "   GUI: ./OpenE2E-GUI_${PLATFORM}/bin/OpenE2E${BINARY_EXT}"
