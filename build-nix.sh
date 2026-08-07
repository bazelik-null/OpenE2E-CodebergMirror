#!/usr/bin/env bash
set -e

nix build .#cli -o result-cli
nix build .#gui -o result-gui

echo "Build complete:"
echo "  CLI: result-cli/bin/OpenE2E"
echo "  GUI: result-gui/bin/OpenE2E"
