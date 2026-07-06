#!/bin/bash

# Setup temporary writeable directories for sandbox compliance
export CARGO_HOME="/tmp/.cargo"
export RUSTUP_HOME="/tmp/.rustup"
# Prioritize pre-installed system Rust paths along with our custom bin folder
export PATH="$CARGO_HOME/bin:/rust/bin:$PATH"

mkdir -p "$CARGO_HOME/bin"
mkdir -p "$RUSTUP_HOME"

# Install only the missing target using the pre-installed rustup/rust components
rustup target add wasm32-unknown-unknown 2>/dev/null || /rust/bin/rustup target add wasm32-unknown-unknown

# Download trunk binary directly to bin path
curl -L --proto '=https' --tlsv1.2 -sSf https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- -C "$CARGO_HOME/bin"

# Compile via dynamic target configuration
trunk build --config examples/basic_app/Trunk.toml --release