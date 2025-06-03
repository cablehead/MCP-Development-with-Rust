#!/bin/bash

# Comprehensive test script for all MCP examples
echo "🧪 Testing All MCP Rust Examples"
echo "================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "${BLUE}🔍 Testing: $test_name${NC}"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if eval "$test_command" > /tmp/test_output.log 2>&1; then
        echo -e "${GREEN}✅ PASSED: $test_name${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ FAILED: $test_name${NC}"
        echo -e "${YELLOW}Error output:${NC}"
        cat /tmp/test_output.log | head -20
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    echo ""
}

# Test compilation of all examples
echo -e "${BLUE}📦 Compilation Tests${NC}"
echo "==================="

run_test "Example 01 - Hello World (compile)" "cargo check --bin example_01_hello_world"
run_test "Example 02 - Calculator (compile)" "cargo check --bin example_02_calculator"
run_test "Example 03 - Text Processor (compile)" "cargo check --bin example_03_text_processor"
run_test "Example 04 - Simple Client (compile)" "cargo check --bin example_04_simple_client"
run_test "Example 05 - Resource Provider (compile)" "cargo check --bin example_05_resource_provider"
run_test "Example 06 - Configurable Server (compile)" "cargo check --bin example_06_configurable_server"
run_test "Example 07 - File Operations (compile)" "cargo check --bin example_07_file_operations"
run_test "Example 08 - HTTP Client (compile)" "cargo check --bin example_08_http_client"
run_test "Example 09 - Database (compile)" "cargo check --bin example_09_database"
run_test "Example 10 - Streaming (compile)" "cargo check --bin example_10_streaming"
run_test "Example 11 - Monitoring (compile)" "cargo check --bin example_11_monitoring"
run_test "Example 12 - Task Queue (compile)" "cargo check --bin example_12_task_queue"
run_test "Example 13 - Auth Service (compile)" "cargo check --bin example_13_auth_service"
run_test "Example 14 - Notification Service (compile)" "cargo check --bin example_14_notification_service"
run_test "Example 15 - Data Pipeline (compile)" "cargo check --bin example_15_data_pipeline"
run_test "Example 16 - Search Service (compile)" "cargo check --bin example_16_search_service"
run_test "Example 17 - Blockchain Integration (compile)" "cargo check --bin example_17_blockchain_integration"
run_test "Example 18 - ML Model Server (compile)" "cargo check --bin example_18_ml_model_server"
run_test "Example 19 - Microservice Gateway (compile)" "cargo check --bin example_19_microservice_gateway"
run_test "Example 20 - Enterprise Server (compile)" "cargo check --bin example_20_enterprise_server"

# Test unit tests for examples that have them
echo -e "${BLUE}🧪 Unit Tests${NC}"
echo "============="

run_test "Example 02 - Calculator (unit tests)" "cargo test --bin example_02_calculator"
run_test "Example 03 - Text Processor (unit tests)" "cargo test --bin example_03_text_processor"
run_test "Example 04 - Simple Client (unit tests)" "cargo test --bin example_04_simple_client"
run_test "Example 05 - Resource Provider (unit tests)" "cargo test --bin example_05_resource_provider"

# Test building examples
echo -e "${BLUE}🔨 Build Tests${NC}"
echo "=============="

run_test "Example 01 - Hello World (build)" "cargo build --bin example_01_hello_world"
run_test "Example 02 - Calculator (build)" "cargo build --bin example_02_calculator"
run_test "Example 03 - Text Processor (build)" "cargo build --bin example_03_text_processor"
run_test "Example 04 - Simple Client (build)" "cargo build --bin example_04_simple_client"
run_test "Example 05 - Resource Provider (build)" "cargo build --bin example_05_resource_provider"
run_test "Example 06 - Configurable Server (build)" "cargo build --bin example_06_configurable_server"
run_test "Example 07 - File Operations (build)" "cargo build --bin example_07_file_operations"
run_test "Example 08 - HTTP Client (build)" "cargo build --bin example_08_http_client"
run_test "Example 09 - Database (build)" "cargo build --bin example_09_database"
run_test "Example 10 - Streaming (build)" "cargo build --bin example_10_streaming"
run_test "Example 11 - Monitoring (build)" "cargo build --bin example_11_monitoring"
run_test "Example 12 - Task Queue (build)" "cargo build --bin example_12_task_queue"
run_test "Example 13 - Auth Service (build)" "cargo build --bin example_13_auth_service"
run_test "Example 14 - Notification Service (build)" "cargo build --bin example_14_notification_service"
run_test "Example 15 - Data Pipeline (build)" "cargo build --bin example_15_data_pipeline"
run_test "Example 16 - Search Service (build)" "cargo build --bin example_16_search_service"
run_test "Example 17 - Blockchain Integration (build)" "cargo build --bin example_17_blockchain_integration"
run_test "Example 18 - ML Model Server (build)" "cargo build --bin example_18_ml_model_server"
run_test "Example 19 - Microservice Gateway (build)" "cargo build --bin example_19_microservice_gateway"
run_test "Example 20 - Enterprise Server (build)" "cargo build --bin example_20_enterprise_server"

# Test running examples in demo mode (these examples have built-in demos)
echo -e "${BLUE}🚀 Runtime Tests${NC}"
echo "==============="

# Check if gtimeout is available (macOS with coreutils), otherwise skip runtime tests
if command -v gtimeout >/dev/null 2>&1; then
    TIMEOUT_CMD="gtimeout"
elif command -v timeout >/dev/null 2>&1; then
    TIMEOUT_CMD="timeout"
else
    echo -e "${YELLOW}⚠️  Timeout command not available, skipping runtime tests${NC}"
    echo -e "${BLUE}🚀 Demo Tests (without timeout)${NC}"
    echo "==============================="
    
    # Run examples that have built-in demos
    run_test "Example 03 - Text Processor (demo)" "cargo run --bin example_03_text_processor"
    run_test "Example 04 - Simple Client (demo)" "cargo run --bin example_04_simple_client"
    run_test "Example 05 - Resource Provider (demo)" "cargo run --bin example_05_resource_provider"
    
    echo -e "${YELLOW}ℹ️  Install coreutils for full runtime testing: brew install coreutils${NC}"
    echo ""
    TIMEOUT_CMD=""
fi

if [ -n "$TIMEOUT_CMD" ]; then
    run_test "Example 01 - Hello World (run demo)" "$TIMEOUT_CMD 3s cargo run --bin example_01_hello_world < /dev/null"
    run_test "Example 02 - Calculator (run demo)" "$TIMEOUT_CMD 3s cargo run --bin example_02_calculator < /dev/null"
    run_test "Example 03 - Text Processor (run demo)" "$TIMEOUT_CMD 3s cargo run --bin example_03_text_processor < /dev/null"
    
    # Test JSON-RPC functionality for examples
    echo -e "${BLUE}🔌 Protocol Tests${NC}"
    echo "================="
    
    # Test tools/list for hello world
    run_test "Example 01 - tools/list" "echo '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}' | $TIMEOUT_CMD 2s cargo run --bin example_01_hello_world --quiet"
    
    # Test calculator tool call
    run_test "Example 02 - calculator tool" "echo '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{\"name\":\"calculator\",\"arguments\":{\"operation\":\"add\",\"a\":5,\"b\":3}}}' | $TIMEOUT_CMD 2s cargo run --bin example_02_calculator --quiet"
fi

# Summary
echo "=========================================="
echo -e "${BLUE}📊 Test Summary${NC}"
echo "=========================================="
echo -e "Total Tests:  ${TOTAL_TESTS}"
echo -e "${GREEN}Passed:       ${PASSED_TESTS}${NC}"
echo -e "${RED}Failed:       ${FAILED_TESTS}${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed! Examples are working correctly.${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed. Please check the errors above.${NC}"
    exit 1
fi

# Cleanup
rm -f /tmp/test_output.log 