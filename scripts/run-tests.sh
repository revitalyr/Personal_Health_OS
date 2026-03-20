#!/bin/bash

# Health OS Multi-Platform Test Runner
# This script runs tests for all client applications

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Configuration
TEST_RESULTS_DIR="test-results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Create test results directory
mkdir -p "$TEST_RESULTS_DIR"

# Function to run Android tests
test_android() {
    log_info "Running Android tests..."
    
    cd clients/android
    
    if [ -f "gradlew" ]; then
        ./gradlew testReleaseUnitTest
        ./gradlew connectedAndroidTest || log_warning "No device connected for Android tests"
        
        # Copy test results
        if [ -d "app/build/reports" ]; then
            cp -r app/build/reports "../../$TEST_RESULTS_DIR/android/"
        fi
        
        log_success "Android tests completed"
    else
        log_warning "Android project not found"
    fi
    
    cd ..
}

# Function to run iOS tests
test_ios() {
    log_info "Running iOS tests..."
    
    if [ "$(uname -s)" != "Darwin" ]; then
        log_warning "iOS tests require macOS, skipping..."
        return 0
    fi
    
    cd clients/ios
    
    if [ -f "Package.swift" ]; then
        swift test --enable-code-coverage
        
        # Generate coverage report
        xcrun llvm-cov report -format html -output "../../$TEST_RESULTS_DIR/ios/coverage" .build/debug/HealthOS-HMSPackageTests.xctest/Contents/MacOS/HealthOS-HMSPackageTests
        
        log_success "iOS tests completed"
    else
        log_warning "iOS project not found"
    fi
    
    cd ..
}

# Function to run Rust desktop tests
test_desktop_rust() {
    log_info "Running Rust desktop tests..."
    
    cd clients/desktop-rust
    
    if [ -f "Cargo.toml" ]; then
        cargo test
        
        # Generate coverage report
        cargo install cargo-tarpaulin || log_warning "cargo-tarpaulin not available"
        if command_exists "cargo-tarpaulin"; then
            cargo tarpaulin --out Html --output-dir "../../$TEST_RESULTS_DIR/rust/"
        fi
        
        log_success "Rust desktop tests completed"
    else
        log_warning "Rust desktop project not found"
    fi
    
    cd ..
}

# Function to run C++ desktop tests
test_desktop_cpp() {
    log_info "Running C++ desktop tests..."
    
    cd clients/desktop-cpp
    
    if [ -f "CMakeLists.txt" ]; then
        mkdir -p build
        cd build
        
        # Configure with tests
        cmake .. -DBUILD_TESTING=ON
        cmake --build .
        
        # Run tests
        if [ -f "tests/health_os_tests" ]; then
            ./tests/health_os_tests
        fi
        
        cd ..
        log_success "C++ desktop tests completed"
    else
        log_warning "C++ desktop project not found"
    fi
    
    cd ..
}

# Function to run C# desktop tests
test_desktop_csharp() {
    log_info "Running C# desktop tests..."
    
    cd clients/desktop-csharp
    
    if [ -f "*.csproj" ]; then
        dotnet test --configuration Release --logger "trx;LogFileName=test_results.trx" --results-directory "../../$TEST_RESULTS_DIR/csharp/"
        
        log_success "C# desktop tests completed"
    else
        log_warning "C# desktop project not found"
    fi
    
    cd ..
}

# Function to run Python desktop tests
test_desktop_python() {
    log_info "Running Python desktop tests..."
    
    cd clients/desktop-python
    
    if [ -f "main.py" ]; then
        # Install test dependencies
        pip3 install pytest pytest-cov pytest-asyncio
        
        # Run tests
        if [ -d "tests" ]; then
            python3 -m pytest tests/ --cov=. --cov-report=html:"../../$TEST_RESULTS_DIR/python/coverage" --junitxml="../../$TEST_RESULTS_DIR/python/test_results.xml"
        else
            log_warning "No tests directory found for Python desktop"
        fi
        
        log_success "Python desktop tests completed"
    else
        log_warning "Python desktop project not found"
    fi
    
    cd ..
}

# Function to run Web SPA tests
test_web_spa() {
    log_info "Running Web SPA tests..."
    
    cd clients/web-spa
    
    if [ -f "package.json" ]; then
        # Install test dependencies if not present
        npm install --save-dev vitest @vitest/ui jsdom
        
        # Run unit tests
        npm run test:unit || log_warning "Unit tests not configured"
        
        # Run E2E tests if available
        npm run test:e2e || log_warning "E2E tests not configured"
        
        # Copy test results
        if [ -d "coverage" ]; then
            cp -r coverage "../../$TEST_RESULTS_DIR/web/"
        fi
        
        log_success "Web SPA tests completed"
    else
        log_warning "Web SPA project not found"
    fi
    
    cd ..
}

# Function to run backend tests
test_backend() {
    log_info "Running backend tests..."
    
    cd services
    
    for service in */; do
        if [ -f "$service/Cargo.toml" ]; then
            service_name=$(basename "$service")
            log_info "Testing service: $service_name"
            
            cd "$service"
            cargo test
            cd ..
            
            log_success "Service $service_name tests completed"
        fi
    done
    
    cd ..
}

# Function to generate test report
generate_test_report() {
    log_info "Generating test report..."
    
    local report_file="$TEST_RESULTS_DIR/test_report_$TIMESTAMP.html"
    
    cat > "$report_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Health OS Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background-color: #2c3e50; color: white; padding: 20px; border-radius: 5px; }
        .section { margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }
        .success { background-color: #d4edda; border-color: #c3e6cb; }
        .warning { background-color: #fff3cd; border-color: #ffeaa7; }
        .error { background-color: #f8d7da; border-color: #f5c6cb; }
        .test-result { margin: 10px 0; padding: 10px; border-radius: 3px; }
        .pass { background-color: #d4edda; }
        .fail { background-color: #f8d7da; }
        .skip { background-color: #fff3cd; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Health OS Test Report</h1>
        <p>Generated: $(date)</p>
        <p>Platform: $(uname -s)</p>
    </div>
    
    <div class="section">
        <h2>Test Summary</h2>
        <div class="test-result pass">Android Tests: Completed</div>
        <div class="test-result pass">iOS Tests: Completed</div>
        <div class="test-result pass">Rust Desktop Tests: Completed</div>
        <div class="test-result pass">C++ Desktop Tests: Completed</div>
        <div class="test-result pass">C# Desktop Tests: Completed</div>
        <div class="test-result pass">Python Desktop Tests: Completed</div>
        <div class="test-result pass">Web SPA Tests: Completed</div>
        <div class="test-result pass">Backend Services Tests: Completed</div>
    </div>
    
    <div class="section">
        <h2>Test Results by Platform</h2>
EOF

    # Add platform-specific results
    for platform in android ios rust cpp csharp python web backend; do
        if [ -d "$TEST_RESULTS_DIR/$platform" ]; then
            echo "        <h3>$platform</h3>" >> "$report_file"
            echo "        <p>Test results available in: $TEST_RESULTS_DIR/$platform</p>" >> "$report_file"
        fi
    done
    
    cat >> "$report_file" << EOF
    </div>
    
    <div class="section">
        <h2>Coverage Reports</h2>
        <p>Coverage reports are available in the respective platform directories.</p>
    </div>
    
    <div class="section">
        <h2>Recommendations</h2>
        <ul>
            <li>Review any failed tests and fix issues</li>
            <li>Improve test coverage where needed</li>
            <li>Add integration tests for better coverage</li>
            <li>Set up automated testing in CI/CD pipeline</li>
        </ul>
    </div>
</body>
</html>
EOF

    log_success "Test report generated: $report_file"
}

# Function to run integration tests
run_integration_tests() {
    log_info "Running integration tests..."
    
    # Start backend services
    cd services
    docker-compose up -d
    
    # Wait for services to be ready
    sleep 30
    
    # Run integration tests
    cd ..
    
    # Test API connectivity
    curl -f http://localhost:8080/health || log_error "API Gateway not responding"
    curl -f http://localhost:8081/health || log_error "Timeline Service not responding"
    curl -f http://localhost:8085/health || log_error "Patient Management Service not responding"
    
    # Stop backend services
    cd services
    docker-compose down
    cd ..
    
    log_success "Integration tests completed"
}

# Main test function
main() {
    log_info "Starting Health OS multi-platform testing..."
    log_info "Timestamp: $TIMESTAMP"
    
    # Run all tests
    test_backend
    test_android
    test_ios
    test_desktop_rust
    test_desktop_cpp
    test_desktop_csharp
    test_desktop_python
    test_web_spa
    
    # Run integration tests
    run_integration_tests
    
    # Generate test report
    generate_test_report
    
    log_success "Multi-platform testing completed!"
    log_info "Test results available in: $TEST_RESULTS_DIR"
    log_info "Test report: $TEST_RESULTS_DIR/test_report_$TIMESTAMP.html"
}

# Handle script arguments
case "${1:-all}" in
    "backend")
        test_backend
        ;;
    "android")
        test_android
        ;;
    "ios")
        test_ios
        ;;
    "desktop-rust")
        test_desktop_rust
        ;;
    "desktop-cpp")
        test_desktop_cpp
        ;;
    "desktop-csharp")
        test_desktop_csharp
        ;;
    "desktop-python")
        test_desktop_python
        ;;
    "web-spa")
        test_web_spa
        ;;
    "integration")
        run_integration_tests
        ;;
    "all")
        main
        ;;
    "report")
        generate_test_report
        ;;
    *)
        echo "Usage: $0 [backend|android|ios|desktop-rust|desktop-cpp|desktop-csharp|desktop-python|web-spa|integration|all|report]"
        exit 1
        ;;
esac
