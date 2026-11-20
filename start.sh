#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

# Opens the app straight from this folder, for trying changes.
# The first time takes a few minutes while everything is built.

if [[ ! -d node_modules ]]; then
    echo "Fetching the window's building blocks. This happens once."
    npm install --ignore-scripts
fi
npm run tauri dev
