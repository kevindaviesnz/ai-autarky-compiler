#!/bin/bash

# 1. Compile the Autarky Compiler (Rust)
echo "🛠️ Building the Autarky Compiler..."
cargo build --release

# 2. Global Installation of the Compiler
echo "📦 Installing the compiler to /usr/local/bin/atk..."
sudo cp target/release/autarky /usr/local/bin/atk

echo "✅ Success! The 'atk' compiler is now universally accessible."