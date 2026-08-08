#!/usr/bin/env bash
set -e

# Store background job PIDs
PIDS=()
BUILD_FAILED=0

# Cleanup function
cleanup() {
    echo ""
    echo "Build interrupted by user" >&2

    # Kill all background jobs
    for pid in "${PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
        fi
    done

    # Wait for them to finish
    wait 2>/dev/null || true

    exit 130
}

trap cleanup SIGINT

# Detect platform
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
    PLATFORM="windows"
    BINARY_EXT=".exe"
else
    PLATFORM="linux"
    BINARY_EXT=""
fi

# Calculate cores for each job
TOTAL_CORES=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
JOBS=2
CORES_PER_JOB=$((TOTAL_CORES / JOBS))

echo "Building for $PLATFORM ($TOTAL_CORES cores, $CORES_PER_JOB per job)"
echo ""

# Create output directories
mkdir -p "OpenE2E-CLI_${PLATFORM}/bin"
mkdir -p "OpenE2E-GUI_${PLATFORM}/bin"

# Build CLI
cargo build --release -j "$CORES_PER_JOB" > cli-build.log 2>&1 &
PIDS+=($!)
echo "Building CLI (see cli-build.log)..."

# Build GUI with features
cargo build --release --features gui -j "$CORES_PER_JOB" > gui-build.log 2>&1 &
PIDS+=($!)
echo "Building GUI (see gui-build.log)..."

# Wait for each job and check status
for pid in "${PIDS[@]}"; do
    if ! wait "$pid"; then
        BUILD_FAILED=1
    fi
done

# Report results
echo ""
if [ $BUILD_FAILED -eq 1 ]; then
    echo "Build failed. See logs:" >&2
    echo "   CLI log: cli-build.log" >&2
    echo "   GUI log: gui-build.log" >&2
    echo "" >&2
    exit 1
fi

echo "Packaging binaries..."

# Copy binaries to output directories
cp "target/release/OpenE2E${BINARY_EXT}" "OpenE2E-CLI_${PLATFORM}/bin/OpenE2E${BINARY_EXT}"
cp "target/release/OpenE2E${BINARY_EXT}" "OpenE2E-GUI_${PLATFORM}/bin/OpenE2E${BINARY_EXT}"

echo ""
echo "Build succeeded:"
echo "   CLI: ./OpenE2E-CLI_${PLATFORM}/bin/OpenE2E${BINARY_EXT}"
echo "   GUI: ./OpenE2E-GUI_${PLATFORM}/bin/OpenE2E${BINARY_EXT}"
