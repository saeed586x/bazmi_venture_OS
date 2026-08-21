#!/bin/bash
# Simple test script

echo "=== TESTING VENTURE CLI ==="

# Build the project
echo "Building project..."
source "$HOME/.cargo/env" && cargo build

if [ $? -eq 0 ]; then
    echo "✅ Build successful"
    
    # Test the CLI
    echo "\nTesting CLI with sample idea..."
    ./target/debug/venture-cli "Create a customer portal"
    
    if [ $? -eq 0 ]; then
        echo "✅ CLI test successful"
    else
        echo "❌ CLI test failed"
    fi
else
    echo "❌ Build failed"
fi
