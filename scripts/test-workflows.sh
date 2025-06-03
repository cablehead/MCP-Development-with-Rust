#!/bin/bash
# GitHub Actions Local Testing Script
# This script helps you test your GitHub Actions workflows locally using act

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if act is installed
if ! command -v act &> /dev/null; then
    print_error "act is not installed. Please install it first:"
    echo "  brew install act"
    exit 1
fi

# Check if Docker is running
if ! docker info &> /dev/null; then
    print_error "Docker is not running. Please start Docker first."
    exit 1
fi

print_status "🎭 GitHub Actions Local Testing Script"
echo ""

# Function to run a workflow job
run_job() {
    local job_name="$1"
    local description="$2"
    
    print_status "Testing: $description"
    echo "Command: act push --job $job_name"
    echo ""
    
    # Try running the job
    if act push --job "$job_name"; then
        print_success "$description completed successfully!"
    else
        print_warning "$description failed with act. This might be due to action caching issues."
        print_status "Common solutions:"
        echo "  1. Clear act cache: rm -rf ~/.cache/act"
        echo "  2. Use GitHub Actions directly for full testing"
        echo "  3. Use act with --action-offline-mode flag"
        return 1
    fi
    echo ""
}

# Main menu
show_menu() {
    echo "🚀 Choose what to test:"
    echo ""
    echo "1) Quick Test (quality checks only)"
    echo "2) Build Test (compile all examples)"  
    echo "3) Security Audit"
    echo "4) Documentation Build"
    echo "5) Full CI Pipeline (all jobs)"
    echo "6) List Available Jobs"
    echo "7) Custom Job"
    echo "q) Quit"
    echo ""
}

# Quick test function
quick_test() {
    print_status "🔍 Running quick quality checks..."
    run_job "quality" "Code Quality (clippy, formatting)"
}

# Build test function
build_test() {
    print_status "🔨 Running build tests..."
    run_job "test" "Build and Test Suite"
}

# Security test function
security_test() {
    print_status "🛡️ Running security audit..."
    run_job "security-audit" "Security Audit"
}

# Documentation test function
docs_test() {
    print_status "📚 Running documentation build..."
    run_job "build-docs" "Documentation Build"
}

# Full CI test function
full_ci_test() {
    print_status "🚀 Running full CI pipeline..."
    print_warning "This will take several minutes and requires significant resources."
    read -p "Continue? (y/N): " -n 1 -r
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        run_job "test" "Build and Test Suite" && \
        run_job "quality" "Code Quality" && \
        run_job "security-audit" "Security Audit" && \
        run_job "build-docs" "Documentation Build"
        
        if [ $? -eq 0 ]; then
            print_success "🎉 Full CI pipeline completed successfully!"
        else
            print_error "❌ CI pipeline failed!"
        fi
    else
        print_status "Cancelled."
    fi
}

# List jobs function
list_jobs() {
    print_status "📋 Available workflow jobs:"
    act --list
}

# Custom job function
custom_job() {
    echo ""
    echo "Available jobs:"
    act --list | grep -E "^\d+" | awk '{print "  - " $2}'
    echo ""
    read -p "Enter job name: " job_name
    
    if [ -n "$job_name" ]; then
        run_job "$job_name" "Custom Job: $job_name"
    else
        print_warning "No job name provided."
    fi
}

# Main loop
main() {
    print_status "Docker status: $(docker info >/dev/null 2>&1 && echo '✅ Running' || echo '❌ Not running')"
    print_status "Act version: $(act --version 2>/dev/null || echo 'Unknown')"
    echo ""
    
    while true; do
        show_menu
        read -p "Choose an option (1-7, q): " choice
        echo ""
        
        case $choice in
            1)
                quick_test
                ;;
            2)
                build_test
                ;;
            3)
                security_test
                ;;
            4)
                docs_test
                ;;
            5)
                full_ci_test
                ;;
            6)
                list_jobs
                ;;
            7)
                custom_job
                ;;
            q|Q)
                print_status "👋 Goodbye!"
                exit 0
                ;;
            *)
                print_warning "Invalid option. Please choose 1-7 or q."
                ;;
        esac
        
        echo ""
        read -p "Press Enter to continue..."
        echo ""
    done
}

# Run main function
main "$@" 