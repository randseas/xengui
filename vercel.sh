#!/bin/bash
set -e

CACHE_DIR="$PWD/.vercel_rust_cache"
export CARGO_HOME="$CACHE_DIR/.cargo"
export RUSTUP_HOME="$CACHE_DIR/.rustup"
export CARGO_TARGET_DIR="$PWD/target"
export PATH="$CARGO_HOME/bin:$PATH"

mkdir -p "$CARGO_HOME/bin"
mkdir -p "$RUSTUP_HOME"

rustup default stable
rustup target add wasm32-unknown-unknown

TRUNK_VERSION="0.21.14"
if ! command -v trunk &> /dev/null; then
    curl -sL "https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz" | tar -xzf - -C "$CARGO_HOME/bin"
    chmod +x "$CARGO_HOME/bin/trunk"
fi

trunk build --config examples/basic_app/Trunk.toml --release