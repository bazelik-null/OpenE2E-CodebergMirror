#!/usr/bin/env bash
set -e

# Store background job PIDs
PIDS=()

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

# Update lock file
echo "Updating flake.lock..."
nix flake update

# Calculate cores for each job
TOTAL_CORES=$(nproc)
JOBS=2
CORES_PER_JOB=$((TOTAL_CORES / JOBS))

echo "Using $CORES_PER_JOB cores per job ($TOTAL_CORES total cores)"
echo ""

# Build with output to log files
nix build .#cli -o OpenE2E-CLI_NixOS --cores "$CORES_PER_JOB" --no-eval-cache > cli-build.log 2>&1 &
PIDS+=($!)
echo "Building CLI (see cli-build.log)..."

nix build .#gui -o OpenE2E-GUI_NixOS --cores "$CORES_PER_JOB" --no-eval-cache > gui-build.log 2>&1 &
PIDS+=($!)
echo "Building GUI (see gui-build.log)..."

# Wait and check if any job failed
if ! wait; then
    echo ""
    echo "One or more builds failed. See logs." >&2
    echo ""
    exit 1
fi

echo ""
echo "Build complete:"
echo "   CLI: ./OpenE2E-CLI_NixOS/bin/OpenE2E"
echo "   GUI: ./OpenE2E-GUI_NixOS/bin/OpenE2E"
