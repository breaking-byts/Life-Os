#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

bold() { printf "\033[1m%s\033[0m\n" "$1"; }

bold "Cleaning Tauri build artifacts..."
rm -rf "$ROOT/src-tauri/target"

if [[ "${CLEAN_NODE_CACHE:-}" == "1" ]]; then
  bold "Cleaning local node caches..."
  rm -rf "$ROOT/node_modules/.vite"
  rm -rf "$ROOT/.tanstack"
fi

if [[ "${CLEAN_CARGO_CACHE:-}" == "1" ]]; then
  bold "Cleaning Cargo caches (global)..."
  rm -rf "$HOME/.cargo/registry" "$HOME/.cargo/git"
fi

bold "Done."
