#!/bin/bash

# Health OS Multi-Platform Build Script
# This script builds all client applications for all platforms

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
BUILD_DIR="build"
DIST_DIR="dist"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
VERSION=$(git describe --tags --always --dirty 2>/dev/null || echo "v1.0.0")

# Create directories
mkdir -p "$BUILD_DIR"
mkdir -p "$DIST_DIR"

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check dependencies
check_dependencies() {
    log_info "Checking dependencies..."
    
    # Required tools
    local tools=("git" "curl" "tar" "gzip")
    
    for tool in "${tools[@]}"; do
        if ! command_exists "$tool"; then
            log_error "Required tool '$tool' is not installed"
            exit 1
        fi
    done
    
    # Platform-specific dependencies
    case "$(uname -s)" in
        Linux*)
            if ! command_exists "docker"; then
                log_warning "Docker not found - some builds may fail"
            fi
            ;;
        Darwin*)
            if ! command_exists "xcodebuild"; then
                log_warning "Xcode not found - iOS build may fail"
            fi
            ;;
        CYGWIN*|MINGW*|MSYS*)
            if ! command_exists "msbuild"; then
                log_warning "MSBuild not found - C# build may fail"
            fi
            ;;
    esac
    
    log_success "Dependencies check completed"
}

# Function to build backend services
build_backend() {
    log_info "Building backend services..."
    
    cd services
    
    for service in */; do
        if [ -f "$service/Cargo.toml" ]; then
            service_name=$(basename "$service")
            log_info "Building service: $service_name"
            
            cd "$service"
            cargo build --release
            cd ..
            
            log_success "Service $service_name built successfully"
        fi
    done
    
    cd ..
    log_success "Backend services build completed"
}

# Function to build Android client
build_android() {
    log_info "Building Android client..."
    
    if ! command_exists "java" || ! command_exists "gradle"; then
        log_error "Java and Gradle are required for Android build"
        return 1
    fi
    
    cd clients/android
    
    # Check if Android SDK is available
    if [ -z "$ANDROID_HOME" ]; then
        log_warning "ANDROID_HOME not set, trying to detect..."
        if [ -d "$HOME/Android/Sdk" ]; then
            export ANDROID_HOME="$HOME/Android/Sdk"
        else
            log_error "Android SDK not found"
            cd ../..
            return 1
        fi
    fi
    
    # Build APK
    ./gradlew assembleRelease
    
    # Copy APK to dist directory
    cp app/build/outputs/apk/release/app-release.apk "../../$DIST_DIR/health-os-android-$VERSION.apk"
    
    cd ../..
    log_success "Android client built successfully"
}

# Function to build iOS client
build_ios() {
    log_info "Building iOS client..."
    
    if [ "$(uname -s)" != "Darwin" ]; then
        log_warning "iOS build requires macOS, skipping..."
        return 0
    fi
    
    if ! command_exists "xcodebuild"; then
        log_error "Xcode is required for iOS build"
        return 1
    fi
    
    cd clients/ios
    
    # Build with Swift Package Manager
    swift build -c release
    
    # Create Xcode archive
    xcodebuild -scheme HealthOS-HMS-App -configuration Release archive -archivePath "$BUILD_DIR/HealthOS-HMS.xcarchive"
    
    # Export archive
    xcodebuild -exportArchive -archivePath "$BUILD_DIR/HealthOS-HMS.xcarchive" -exportPath "../../$DIST_DIR/health-os-ios-$VERSION" -exportOptionsPlist ExportOptions.plist
    
    cd ../..
    log_success "iOS client built successfully"
}

# Function to build Rust desktop client
build_desktop_rust() {
    log_info "Building Rust desktop client..."
    
    if ! command_exists "cargo"; then
        log_error "Rust/Cargo is required for Rust desktop build"
        return 1
    fi
    
    cd clients/desktop-rust
    
    # Install Tauri CLI if not present
    if ! command_exists "cargo-tauri"; then
        log_info "Installing Tauri CLI..."
        cargo install tauri-cli
    fi
    
    # Build for current platform
    cargo tauri build
    
    # Copy binaries based on platform
    case "$(uname -s)" in
        Linux*)
            cp src-tauri/target/release/bundle/deb/*.deb "../../$DIST_DIR/"
            cp src-tauri/target/release/bundle/appimage/*.AppImage "../../$DIST_DIR/"
            ;;
        Darwin*)
            cp -r src-tauri/target/release/bundle/macos/*.app "../../$DIST_DIR/"
            ;;
        CYGWIN*|MINGW*|MSYS*)
            cp src-tauri/target/release/bundle/msi/*.msi "../../$DIST_DIR/"
            ;;
    esac
    
    cd ../..
    log_success "Rust desktop client built successfully"
}

# Function to build C++ desktop client
build_desktop_cpp() {
    log_info "Building C++ desktop client..."
    
    if ! command_exists "cmake"; then
        log_error "CMake is required for C++ desktop build"
        return 1
    fi
    
    cd clients/desktop-cpp
    
    # Create build directory
    mkdir -p build
    cd build
    
    # Configure and build
    cmake ..
    cmake --build . --config Release
    
    # Copy executable
    case "$(uname -s)" in
        Linux*)
            cp health-os-hms "../../../$DIST_DIR/health-os-desktop-cpp-linux-$VERSION"
            ;;
        Darwin*)
            cp health-os-hms "../../../$DIST_DIR/health-os-desktop-cpp-macos-$VERSION"
            ;;
        CYGWIN*|MINGW*|MSYS*)
            cp Release/health-os-hms.exe "../../../$DIST_DIR/health-os-desktop-cpp-windows-$VERSION.exe"
            ;;
    esac
    
    cd ../../..
    log_success "C++ desktop client built successfully"
}

# Function to build C# desktop client
build_desktop_csharp() {
    log_info "Building C# desktop client..."
    
    if ! command_exists "dotnet"; then
        log_error ".NET SDK is required for C# desktop build"
        return 1
    fi
    
    cd clients/desktop-csharp
    
    # Restore dependencies
    dotnet restore
    
    # Build
    dotnet build --configuration Release
    
    # Publish
    dotnet publish --configuration Release --output "../$DIST_DIR/health-os-desktop-csharp-$VERSION"
    
    cd ../..
    log_success "C# desktop client built successfully"
}

# Function to build Python desktop client
build_desktop_python() {
    log_info "Building Python desktop client..."
    
    if ! command_exists "python3"; then
        log_error "Python 3 is required for Python desktop build"
        return 1
    fi
    
    cd clients/desktop-python
    
    # Install dependencies
    pip3 install -r requirements.txt
    pip3 install pyinstaller
    
    # Build executable
    pyinstaller --onefile --windowed main.py
    
    # Copy executable
    case "$(uname -s)" in
        Linux*)
            cp dist/main "../../$DIST_DIR/health-os-desktop-python-linux-$VERSION"
            ;;
        Darwin*)
            cp dist/main "../../$DIST_DIR/health-os-desktop-python-macos-$VERSION"
            ;;
        CYGWIN*|MINGW*|MSYS*)
            cp dist/main.exe "../../$DIST_DIR/health-os-desktop-python-windows-$VERSION.exe"
            ;;
    esac
    
    cd ../..
    log_success "Python desktop client built successfully"
}

# Function to build Web SPA
build_web_spa() {
    log_info "Building Web SPA..."
    
    if ! command_exists "node"; then
        log_error "Node.js is required for Web SPA build"
        return 1
    fi
    
    cd clients/web-spa
    
    # Install dependencies
    npm ci
    
    # Build
    npm run build
    
    # Copy build output
    cp -r dist "../../$DIST_DIR/health-os-web-spa-$VERSION"
    
    cd ../..
    log_success "Web SPA built successfully"
}

# Function to create release package
create_release_package() {
    log_info "Creating release package..."
    
    cd "$DIST_DIR"
    
    # Create archive for each platform
    tar -czf "health-os-android-$VERSION.tar.gz" health-os-android-$VERSION.apk
    
    if [ -d "health-os-ios-$VERSION" ]; then
        tar -czf "health-os-ios-$VERSION.tar.gz" health-os-ios-$VERSION/
    fi
    
    # Desktop clients
    for platform in linux macos windows; do
        for lang in rust cpp python; do
            if [ -f "health-os-desktop-$lang-$platform-$VERSION" ] || [ -f "health-os-desktop-$lang-$platform-$VERSION.exe" ]; then
                tar -czf "health-os-desktop-$lang-$platform-$VERSION.tar.gz" health-os-desktop-$lang-$platform-$VERSION*
            fi
        done
    done
    
    # C# and Web SPA
    if [ -d "health-os-desktop-csharp-$VERSION" ]; then
        tar -czf "health-os-desktop-csharp-$VERSION.tar.gz" health-os-desktop-csharp-$VERSION/
    fi
    
    if [ -d "health-os-web-spa-$VERSION" ]; then
        tar -czf "health-os-web-spa-$VERSION.tar.gz" health-os-web-spa-$VERSION/
    fi
    
    # Create checksums
    sha256sum *.tar.gz > checksums.txt
    
    cd ..
    log_success "Release package created successfully"
}

# Function to clean up
cleanup() {
    log_info "Cleaning up..."
    rm -rf "$BUILD_DIR"
    log_success "Cleanup completed"
}

# Main build function
main() {
    log_info "Starting Health OS multi-platform build..."
    log_info "Version: $VERSION"
    log_info "Timestamp: $TIMESTAMP"
    
    # Check dependencies
    check_dependencies
    
    # Build all components
    build_backend
    build_android
    build_ios
    build_desktop_rust
    build_desktop_cpp
    build_desktop_csharp
    build_desktop_python
    build_web_spa
    
    # Create release package
    create_release_package
    
    # Clean up
    cleanup
    
    log_success "Multi-platform build completed successfully!"
    log_info "All artifacts are available in: $DIST_DIR"
}

# Handle script arguments
case "${1:-all}" in
    "backend")
        check_dependencies
        build_backend
        ;;
    "android")
        check_dependencies
        build_android
        ;;
    "ios")
        check_dependencies
        build_ios
        ;;
    "desktop-rust")
        check_dependencies
        build_desktop_rust
        ;;
    "desktop-cpp")
        check_dependencies
        build_desktop_cpp
        ;;
    "desktop-csharp")
        check_dependencies
        build_desktop_csharp
        ;;
    "desktop-python")
        check_dependencies
        build_desktop_python
        ;;
    "web-spa")
        check_dependencies
        build_web_spa
        ;;
    "all")
        main
        ;;
    "clean")
        cleanup
        ;;
    *)
        echo "Usage: $0 [backend|android|ios|desktop-rust|desktop-cpp|desktop-csharp|desktop-python|web-spa|all|clean]"
        exit 1
        ;;
esac
