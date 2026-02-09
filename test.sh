#!/bin/bash

# Test script for snapshot.nvim

echo "Testing snapshot.nvim..."
echo ""

# Test 1: Check if generator exists
if [ -f "./generator/target/release/snapshot-generator" ]; then
    echo "✓ Generator binary found"
else
    echo "✗ Generator binary not found"
    echo "  Run: make build"
    exit 1
fi

# Test 2: Test generator with sample JSON
echo "✓ Testing image generation..."
cat test_input.json | ./generator/target/release/snapshot-generator > /tmp/test_output.txt 2>&1
if [ $? -eq 0 ]; then
    OUTPUT_PATH=$(cat /tmp/test_output.txt | tr -d '[:space:]')
    if [ -f "$OUTPUT_PATH" ]; then
        echo "✓ Image generated successfully at: $OUTPUT_PATH"
        FILE_SIZE=$(ls -lh "$OUTPUT_PATH" | awk '{print $5}')
        echo "  File size: $FILE_SIZE"
    else
        echo "✗ Image file not created"
        exit 1
    fi
else
    echo "✗ Generator failed"
    cat /tmp/test_output.txt
    exit 1
fi

echo ""
echo "All tests passed! ✓"
echo ""
echo "To test in Neovim:"
echo "  1. Open a Lua file"
echo "  2. Select some code in visual mode (V)"
echo "  3. Run :Snapshot"
echo "  4. Check ~/snapshot.png"
