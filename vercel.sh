#!/bin/bash

# Setup temporary writeable directories for sandbox compliance
export CARGO_HOME="/tmp/.cargo"
export RUSTUP_HOME="/tmp/.rustup"
export PATH="$CARGO_HOME/bin:/rust/bin:$PATH"

mkdir -p "$CARGO_HOME/bin"
mkdir -p "$RUSTUP_HOME"

# Explicitly point target and build directories to a place Vercel caches
# Vercel preserves the project directory between builds, so we use it for caching
export CARGO_TARGET_DIR="$PWD/target"

rustup default stable
rustup target add wasm32-unknown-unknown

# Install trunk only if the binary does not already exist in cache
if ! command -v trunk &> /dev/null; then
    echo "Trunk not found in cache. Installing from source..."
    cargo install trunk --version 0.21.14 --root /tmp/.cargo
else
    echo "Trunk found in cache! Skipping installation."
fi

# Compile via dynamic target configuration
trunk build --config examples/basic_app/Trunk.toml --release