#!/bin/bash

# Install Rustup and WASM target
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
rustup target add wasm32-unknown-unknown

# Install Trunk
curl -L --proto '=https' --tlsv1.2 -sSf https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- -C $HOME/.cargo/bin

# Execute compilation
trunk build examples/basic_app/index.html --release