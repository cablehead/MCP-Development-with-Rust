#!/bin/bash

# Test script for example_01_hello_world
echo "🧪 Testing Example 01: Hello World MCP Server"
echo "=============================================="

# Build the example
echo "📦 Building example..."
cargo build --bin example_01_hello_world --quiet

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build successful!"

# Test 1: List tools
echo ""
echo "🔍 Test 1: Listing available tools"
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | timeout 5s cargo run --bin example_01_hello_world --quiet > /tmp/test1_output.json 2>/dev/null

if [ $? -eq 0 ]; then
    echo "✅ tools/list test passed"
    echo "📄 Response:"
    cat /tmp/test1_output.json | jq '.' 2>/dev/null || cat /tmp/test1_output.json
else
    echo "❌ tools/list test failed"
fi

# Test 2: Call greeting tool
echo ""
echo "🔍 Test 2: Calling greeting tool"
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"greeting","arguments":{"name":"Rust Developer"}}}' | timeout 5s cargo run --bin example_01_hello_world --quiet > /tmp/test2_output.json 2>/dev/null

if [ $? -eq 0 ]; then
    echo "✅ greeting tool test passed"
    echo "📄 Response:"
    cat /tmp/test2_output.json | jq '.' 2>/dev/null || cat /tmp/test2_output.json
else
    echo "❌ greeting tool test failed"
fi

# Cleanup
rm -f /tmp/test1_output.json /tmp/test2_output.json

echo ""
echo "🎉 Example 01 testing completed!" 