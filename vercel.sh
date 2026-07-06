#!/bin/bash

export CARGO_HOME="/tmp/.cargo"
export RUSTUP_HOME="/tmp/.rustup"
export PATH="$CARGO_HOME/bin:$PATH"

mkdir -p "$CARGO_HOME/bin"
mkdir -p "$RUSTUP_HOME"

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path

source "$CARGO_HOME/env"
rustup target add wasm32-unknown-unknown

curl -L --proto '=https' --tlsv1.2 -sSf https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- -C "$CARGO_HOME/bin"

trunk build --config examples/basic_app/Trunk.toml --release