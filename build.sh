#!/bin/bash

# Market Simulator Build Script

echo "Building Market Simulator..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Rust is not installed. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Build the project
cargo build --release

if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "Run the market simulator with: cargo run"
    echo "Or use the optimized version: ./target/release/market-sim"
else
    echo "Build failed!"
    exit 1
fi
