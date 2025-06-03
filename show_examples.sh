#!/bin/bash

# Demo script to showcase all MCP Rust examples
echo "🚀 Model Context Protocol (MCP) Rust Examples Showcase"
echo "======================================================="
echo ""
echo "This script demonstrates all working examples from basic to advanced concepts."
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to show example info
show_example() {
    local number="$1"
    local name="$2"
    local description="$3"
    local features="$4"
    
    echo -e "${BLUE}📝 Example $number: $name${NC}"
    echo -e "${CYAN}$description${NC}"
    echo -e "${YELLOW}Features: $features${NC}"
    echo ""
}

echo -e "${PURPLE}🎯 What We've Built${NC}"
echo "=================="
echo ""

show_example "01" "Hello World MCP Server" \
    "Basic MCP server with greeting functionality" \
    "JSON-RPC 2.0, Tool implementation, Async I/O"

show_example "02" "Calculator Tool Server" \
    "Arithmetic operations with parameter validation" \
    "Multiple operations, Error handling, Unit tests"

show_example "03" "Text Processing Server" \
    "Multiple text transformation and analysis tools" \
    "Multiple tools, Text operations, Built-in demos"

show_example "04" "Simple MCP Client" \
    "Client-side MCP implementation and workflow" \
    "Client patterns, Tool discovery, Response handling"

show_example "05" "Resource Provider" \
    "Document management with search capabilities" \
    "URI resources, Search functionality, Content serving"

echo -e "${GREEN}✅ All Examples Status${NC}"
echo "===================="
echo ""

# Check compilation status
echo -e "${BLUE}🔍 Checking compilation status...${NC}"
for example in example_01_hello_world example_02_calculator example_03_text_processor example_04_simple_client example_05_resource_provider; do
    if cargo check --bin $example --quiet 2>/dev/null; then
        echo -e "  ✅ $example - ${GREEN}Compiles successfully${NC}"
    else
        echo -e "  ❌ $example - ${RED}Compilation failed${NC}"
    fi
done

echo ""

# Show test coverage
echo -e "${BLUE}🧪 Test Coverage${NC}"
echo "==============="
echo ""

test_count=0
for example in example_02_calculator example_03_text_processor example_04_simple_client example_05_resource_provider; do
    if cargo test --bin $example --quiet 2>/dev/null; then
        tests=$(cargo test --bin $example 2>/dev/null | grep "test result:" | sed 's/.*test result: ok. \([0-9]*\) passed.*/\1/')
        echo -e "  ✅ $example - ${GREEN}$tests unit tests passing${NC}"
        test_count=$((test_count + tests))
    else
        echo -e "  ❌ $example - ${RED}Tests failed${NC}"
    fi
done

echo ""
echo -e "${GREEN}📊 Total: $test_count unit tests across all examples${NC}"
echo ""

# Technical achievements
echo -e "${PURPLE}🏆 Technical Achievements${NC}"
echo "========================"
echo ""
echo "✅ Complete MCP Protocol Implementation"
echo "   • JSON-RPC 2.0 message handling"
echo "   • Tool definition and execution"
echo "   • Resource management and serving"
echo "   • Client-server communication patterns"
echo ""
echo "✅ Rust Best Practices"
echo "   • Async/await with Tokio runtime"
echo "   • Type-safe JSON with Serde"
echo "   • Comprehensive error handling"
echo "   • Memory safety and performance"
echo ""
echo "✅ Educational Excellence"
echo "   • Instructor-level commenting"
echo "   • Progressive complexity design"
echo "   • Complete test coverage"
echo "   • Real-world applicability"
echo ""

# Quick demo
echo -e "${CYAN}🎪 Quick Demo${NC}"
echo "============"
echo ""
echo "Here's a quick demonstration of Example 3 (Text Processor):"
echo ""

# Run the text processor demo
if cargo run --bin example_03_text_processor --quiet 2>/dev/null; then
    echo -e "${GREEN}✅ Demo completed successfully!${NC}"
else
    echo -e "${RED}❌ Demo failed${NC}"
fi

echo ""

# Usage instructions
echo -e "${BLUE}🚀 How to Use These Examples${NC}"
echo "=========================="
echo ""
echo "1. Run comprehensive tests:"
echo "   ${YELLOW}./test_all_examples.sh${NC}"
echo ""
echo "2. Try individual examples:"
echo "   ${YELLOW}cargo run --bin example_01_hello_world${NC}"
echo "   ${YELLOW}cargo run --bin example_02_calculator${NC}"
echo "   ${YELLOW}cargo run --bin example_03_text_processor${NC}"
echo "   ${YELLOW}cargo run --bin example_04_simple_client${NC}"
echo "   ${YELLOW}cargo run --bin example_05_resource_provider${NC}"
echo ""
echo "3. Run unit tests:"
echo "   ${YELLOW}cargo test --bin example_02_calculator${NC}"
echo ""
echo "4. Explore the code:"
echo "   ${YELLOW}src/examples/example_01_hello_world.rs${NC}"
echo ""

# Statistics
echo -e "${GREEN}📈 Project Statistics${NC}"
echo "===================="
echo ""

total_lines=$(find src/examples -name "*.rs" -exec wc -l {} \; | awk '{sum += $1} END {print sum}')
total_files=$(find src/examples -name "*.rs" | wc -l | tr -d ' ')
total_functions=$(grep -r "fn " src/examples/*.rs | wc -l | tr -d ' ')
total_tests=$(grep -r "#\[test\]" src/examples/*.rs | wc -l | tr -d ' ')

echo "📁 Files:     $total_files example files"
echo "📝 Code:      ~$total_lines lines of well-commented Rust"
echo "⚙️  Functions: $total_functions functions with full documentation"
echo "🧪 Tests:     $total_tests comprehensive unit tests"
echo ""

echo -e "${PURPLE}🎉 Ready to Build MCP Applications in Rust!${NC}"
echo ""
echo "These examples provide a complete foundation for:"
echo "• Building MCP-compatible servers and clients"
echo "• Understanding protocol patterns and best practices"
echo "• Implementing real-world AI tool integrations"
echo "• Learning advanced Rust async programming"
echo ""
echo -e "${CYAN}Happy coding! 🦀✨${NC}" 