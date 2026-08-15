#!/usr/bin/env bash
set -e

# Update lock file
echo "Updating flake.lock..."
nix flake update

# Build with output to log files
echo "Building CLI..."
nix build .#cli -o OpenE2E-CLI_NixOS

echo "Building GUI..."
nix build .#gui -o OpenE2E-GUI_NixOS

# Report results
echo ""
echo "Build succeeded:"
echo "   CLI: ./OpenE2E-CLI_NixOS/bin/OpenE2E"
echo "   GUI: ./OpenE2E-GUI_NixOS/bin/OpenE2E"
