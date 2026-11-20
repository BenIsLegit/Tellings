#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

# Builds the app and the pvc command-line tool, after running every check.
#
#   ./build.sh            a test build: version 0.1.0-a1b2c3d, the program only
#   ./build.sh --release  the real thing: version 0.1.0, with installers
#
# The version lives in package.json. Keep src-tauri/tauri.conf.json and the
# root Cargo.toml the same when you change it.

BASE_VERSION=$(node -p "require('./package.json').version")

if [[ "${1:-}" == "--release" ]]; then
    FULL_VERSION="${BASE_VERSION}"
    BUNDLE_FLAGS=""
    echo "Building RELEASE version: ${FULL_VERSION}"
else
    COMMIT_HASH=$(git rev-parse --short HEAD)
    FULL_VERSION="${BASE_VERSION}-${COMMIT_HASH}"
    BUNDLE_FLAGS="--no-bundle"
    echo "Building TEST version: ${FULL_VERSION}"
fi

if [[ ! -d node_modules ]]; then
    echo "=== Fetching the window's building blocks ==="
    npm install --ignore-scripts
fi

echo "=== Checking the window ==="
npm run check
npm test

echo "=== Checking the engine ==="
cargo test -p pvc-engine -p pvc
cargo check -p prayer-video-creator

echo "=== Building the command-line tool ==="
cargo build --release -p pvc

echo "=== Building the app (v${FULL_VERSION}) ==="
echo "{\"version\":\"${FULL_VERSION}\"}" > .tauri-build-config.json
trap 'rm -f .tauri-build-config.json' EXIT
# shellcheck disable=SC2086
npm run tauri build -- --config .tauri-build-config.json ${BUNDLE_FLAGS}

echo
echo "=== Done (v${FULL_VERSION}) ==="
echo "The app:               target/release/prayer-video-creator"
echo "The command-line tool: target/release/pvc"
if [[ "${1:-}" == "--release" ]]; then
    echo "The installers:        target/release/bundle/"
fi
