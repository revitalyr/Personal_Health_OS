#!/bin/bash
# Security and quality checks for medical system deployment
# This script runs AddressSanitizer, ThreadSanitizer, and clippy with deny warnings

set -e

echo "🔒 Running security and quality checks for Health OS deployment..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to run a check and report result
run_check() {
    local name="$1"
    local command="$2"
    
    echo -e "${YELLOW}Running: $name${NC}"
    if eval "$command"; then
        echo -e "${GREEN}✓ $name passed${NC}"
    else
        echo -e "${RED}✗ $name failed${NC}"
        exit 1
    fi
}

# Check 1: Clippy with deny warnings
echo -e "\n${YELLOW}=== Clippy with deny warnings ===${NC}"
run_check "Clippy (deny warnings)" "cargo clippy --all-targets --all-features -- -D warnings -D clippy::all -D clippy::pedantic -A clippy::too_many_arguments -A clippy::missing_errors_doc"

# Check 2: Cargo fmt check
echo -e "\n${YELLOW}=== Code formatting check ===${NC}"
run_check "Cargo fmt" "cargo fmt --all -- --check"

# Check 3: AddressSanitizer (Linux only)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo -e "\n${YELLOW}=== AddressSanitizer check ===${NC}"
    export RUSTFLAGS="-Z sanitizer=address"
    export ASAN_OPTIONS="detect_leaks=1:halt_on_error=0"
    run_check "AddressSanitizer" "cargo test --all --all-features --target x86_64-unknown-linux-gnu"
    unset RUSTFLAGS
    unset ASAN_OPTIONS
else
    echo -e "${YELLOW}AddressSanitizer skipped (Linux only)${NC}"
fi

# Check 4: ThreadSanitizer (Linux only)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo -e "\n${YELLOW}=== ThreadSanitizer check ===${NC}"
    export RUSTFLAGS="-Z sanitizer=thread"
    run_check "ThreadSanitizer" "cargo test --all --all-features --target x86_64-unknown-linux-gnu"
    unset RUSTFLAGS
else
    echo -e "${YELLOW}ThreadSanitizer skipped (Linux only)${NC}"
fi

# Check 5: Cargo audit for security vulnerabilities
echo -e "\n${YELLOW}=== Security audit ===${NC}"
if command -v cargo-audit &> /dev/null; then
    run_check "Cargo audit" "cargo audit"
else
    echo -e "${YELLOW}cargo-audit not installed, skipping${NC}"
    echo "Install with: cargo install cargo-audit"
fi

# Check 6: Cargo outdated check
echo -e "\n${YELLOW}=== Dependency check ===${NC}"
if command -v cargo-outdated &> /dev/null; then
    run_check "Cargo outdated" "cargo outdated --exit-code 1"
else
    echo -e "${YELLOW}cargo-outdated not installed, skipping${NC}"
    echo "Install with: cargo install cargo-outdated"
fi

echo -e "\n${GREEN}✓ All security and quality checks passed!${NC}"
echo "Ready for deployment."
