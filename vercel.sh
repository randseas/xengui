#!/bin/bash

# Setup temporary writeable directories for sandbox compliance
export CARGO_HOME="/tmp/.cargo"
export RUSTUP_HOME="/tmp/.rustup"
export PATH="$CARGO_HOME/bin:/rust/bin:$PATH"

mkdir -p "$CARGO_HOME/bin"
mkdir -p "$RUSTUP_HOME"

# Configure the default toolchain explicitly to fix the rustup selection error
rustup default stable
rustup target add wasm32-unknown-unknown

# Build trunk from source to prevent GLIBC version mismatch errors
cargo install trunk --version 0.21.14

# Compile via dynamic target configuration
trunk build --config examples/basic_app/Trunk.toml --release