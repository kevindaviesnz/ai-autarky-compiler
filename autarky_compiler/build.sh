#!/bin/bash

# Autarky Native Build Driver
# Usage: ./build.sh <filename.aut>

if [ -z "$1" ]; then
    echo "Usage: ./build.sh <filename.aut>"
    exit 1
fi

INPUT_FILE=$1
OUTPUT_BIN="autarky_native"

echo "🛠️  Step 1: Running Autarky Compiler..."
# We run the Rust compiler to generate output.ll
cargo run --release -- --file "$INPUT_FILE"

if [ $? -ne 0 ]; then
    echo "❌ Rust Compilation Failed."
    exit 1
fi

echo "🔗 Step 2: Linking with LLVM and C Runtime..."
# -O3 enables the LLVM optimizations (including Tail Call Optimization)
# We link your generated IR with the C runtime
clang -O3 runtime.c output.ll -o "$OUTPUT_BIN"

if [ $? -ne 0 ]; then
    echo "❌ Linking Failed."
    exit 1
fi

echo "🚀 Step 3: Executing Native Binary..."
echo "====================================="
./"$OUTPUT_BIN"
echo "====================================="

# Optional: Cleanup the intermediate IR file
# rm output.ll
