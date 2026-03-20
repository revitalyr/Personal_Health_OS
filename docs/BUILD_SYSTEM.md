# Health OS Multi-Platform Build System

## Overview

This build system automates the compilation, testing, and packaging of all Health OS client applications across multiple platforms. It supports both local development builds and CI/CD automation.

## Supported Platforms

### Mobile Applications
- **Android** - Java/Kotlin with Gradle
- **iOS** - Swift with Swift Package Manager

### Desktop Applications
- **Rust** - Tauri-based cross-platform client
- **C++** - Qt6-based native client
- **C#** - WPF/.NET 7 Windows client
- **Python** - Tkinter cross-platform client

### Web Applications
- **Web SPA** - Vue.js with Vite

## Build System Components

### 1. GitHub Actions CI/CD (`.github/workflows/build-all-platforms.yml`)

**Features:**
- Matrix builds across multiple OS (Linux, Windows, macOS)
- Parallel execution for faster builds
- Automated testing and security scanning
- Artifact management and release creation
- Docker containerization for backend services

**Workflows:**
- **Backend Services** - Rust microservices with Docker
- **Mobile Clients** - Android and iOS builds
- **Desktop Clients** - Cross-platform desktop applications
- **Web Client** - Vue.js SPA build and deployment
- **Security Scanning** - Trivy vulnerability scanner
- **Release Management** - Automated packaging and distribution

### 2. Local Build Script (`scripts/build-all-platforms.sh`)

**Features:**
- Cross-platform dependency checking
- Automated build for all platforms
- Platform-specific optimizations
- Error handling and logging
- Release package creation
- Checksum generation

**Usage:**
```bash
# Build all platforms
./scripts/build-all-platforms.sh all

# Build specific platform
./scripts/build-all-platforms.sh android
./scripts/build-all-platforms.sh ios
./scripts/build-all-platforms.sh desktop-rust

# Clean build artifacts
./scripts/build-all-platforms.sh clean
```

## Platform-Specific Requirements

### Android Build Requirements
- **Java 17+** (JDK)
- **Android SDK** (API level 24+)
- **Gradle** 8.0+
- **Android Build Tools**

### iOS Build Requirements
- **macOS** (required for iOS builds)
- **Xcode** 14.0+
- **Swift** 5.9+
- **iOS Simulator** (for testing)

### Rust Desktop Requirements
- **Rust** 1.75+
- **Tauri CLI** (`cargo install tauri-cli`)
- **Node.js** 18+ (for Tauri frontend)
- **Platform-specific dependencies:**
  - Linux: `libwebkit2gtk-4.0-dev`, `libgtk-3-dev`
  - macOS: Xcode Command Line Tools
  - Windows: Visual Studio Build Tools

### C++ Desktop Requirements
- **CMake** 3.20+
- **Qt6** Development Libraries
- **C++17** compatible compiler
- **Platform-specific:**
  - Linux: `qt6-base-dev`, `qt6-tools-dev`
  - macOS: Homebrew Qt6 (`brew install qt@6`)
  - Windows: Qt6 installer or vcpkg

### C# Desktop Requirements
- **.NET SDK** 7.0+
- **Visual Studio** 2022 or VS Code with C# extension
- **Windows SDK** (for WPF applications)

### Python Desktop Requirements
- **Python** 3.11+
- **PyInstaller** (`pip install pyinstaller`)
- **Platform dependencies:**
  - Linux: `python3-tk`, `python3-dev`
  - macOS: Python 3 with Tkinter support
  - Windows: Python 3.11+ installer

### Web SPA Requirements
- **Node.js** 18+
- **npm** or yarn package manager
- **Modern web browser** for testing

## Build Configuration Files

### Android (`clients/android/app/build.gradle`)
```gradle
android {
    compileSdk 34
    defaultConfig {
        applicationId "com.healthos.hms"
        minSdk 24
        targetSdk 34
    }
    buildTypes {
        release {
            minifyEnabled false
            proguardFiles getDefaultProguardFile('proguard-android-optimize.txt')
        }
    }
}
```

### iOS (`clients/ios/Package.swift`)
```swift
let package = Package(
    name: "HealthOS-HMS",
    platforms: [.iOS(.v16), .macOS(.v13)],
    products: [
        .library(name: "HealthOS-HMS", targets: ["HealthOS-HMS"]),
        .executable(name: "HealthOS-HMS-App", targets: ["HealthOS-HMS-App"])
    ]
)
```

### Rust Desktop (`clients/desktop-rust/Cargo.toml`)
```toml
[package]
name = "health-os-hms"
version = "1.0.0"
edition = "2021"

[dependencies]
tauri = { version = "1.4", features = ["api-all"] }
tokio = { version = "1.35", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
```

### C++ Desktop (`clients/desktop-cpp/CMakeLists.txt`)
```cmake
cmake_minimum_required(VERSION 3.20)
project(HealthOS-HMS)

find_package(Qt6 REQUIRED COMPONENTS Core Widgets Network)

add_executable(health-os-hms src/main.cpp)
target_link_libraries(health-os-hms Qt6::Core Qt6::Widgets Qt6::Network)
```

### C# Desktop (`clients/desktop-csharp/HealthOS.HMS.Desktop.csproj`)
```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>WinExe</OutputType>
    <TargetFramework>net7.0-windows</TargetFramework>
    <UseWPF>true</UseWPF>
  </PropertyGroup>
</Project>
```

### Python Desktop (`clients/desktop-python/requirements.txt`)
```txt
aiohttp>=3.8.0
tkinter
matplotlib>=3.7.0
pillow>=10.0.0
pyinstaller>=5.0.0
```

### Web SPA (`clients/web-spa/package.json`)
```json
{
  "name": "health-os-hms-web",
  "version": "1.0.0",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  }
}
```

## CI/CD Pipeline Structure

### Build Matrix
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest]
    include:
      - os: ubuntu-latest
        platform: linux
      - os: windows-latest
        platform: windows
      - os: macos-latest
        platform: macos
```

### Artifact Management
- **Android APK** - `health-os-android-v1.0.0.apk`
- **iOS Archive** - `health-os-ios-v1.0.0.tar.gz`
- **Rust Desktop** - Platform-specific bundles
- **C++ Desktop** - Native executables
- **C# Desktop** - Published application
- **Python Desktop** - PyInstaller executables
- **Web SPA** - Static web assets

### Security Features
- **Trivy Scanning** - Vulnerability detection
- **Dependency Checks** - Package security analysis
- **Code Signing** - Application signing (production)
- **Checksums** - SHA-256 verification

## Local Development Workflow

### 1. Environment Setup
```bash
# Clone repository
git clone https://github.com/revitalyr/Personal_Health_OS.git
cd Personal_Health_OS

# Install build dependencies
./scripts/install-dependencies.sh
```

### 2. Build Specific Platform
```bash
# Build Android app
./scripts/build-all-platforms.sh android

# Build iOS app (macOS only)
./scripts/build-all-platforms.sh ios

# Build Rust desktop
./scripts/build-all-platforms.sh desktop-rust
```

### 3. Test Applications
```bash
# Run Android tests
cd clients/android && ./gradlew test

# Run iOS tests
cd clients/ios && swift test

# Run Rust tests
cd clients/desktop-rust && cargo test
```

### 4. Package for Distribution
```bash
# Create release packages
./scripts/build-all-platforms.sh all

# Generate checksums
cd dist && sha256sum * > checksums.txt
```

## Deployment Strategies

### 1. Development Deployment
- **Local builds** for testing
- **Development servers** for staging
- **Feature branches** for testing new features

### 2. Production Deployment
- **GitHub Releases** for public distribution
- **App Stores** for mobile applications
- **Website hosting** for web SPA
- **Package managers** for desktop applications

### 3. Enterprise Deployment
- **Private repositories** for source code
- **Internal CI/CD** for custom builds
- **Container registries** for Docker images
- **License management** for enterprise features

## Troubleshooting

### Common Issues

#### Android Build Failures
```bash
# Check Android SDK installation
echo $ANDROID_HOME

# Clean Gradle cache
./gradlew clean

# Rebuild project
./gradlew assembleRelease
```

#### iOS Build Failures
```bash
# Check Xcode installation
xcode-select --print-path

# Clean build cache
rm -rf ~/Library/Developer/Xcode/DerivedData

# Rebuild project
swift build -c release
```

#### Rust Build Failures
```bash
# Update Rust toolchain
rustup update

# Clean cargo cache
cargo clean

# Rebuild project
cargo tauri build
```

#### C++ Build Failures
```bash
# Check Qt installation
qmake --version

# Clean build directory
rm -rf build

# Rebuild project
mkdir build && cd build && cmake .. && make
```

### Performance Optimization

#### Build Caching
- **GitHub Actions Cache** - Dependency caching
- **Local Caches** - Platform-specific build caches
- **Docker Layers** - Optimized Docker builds

#### Parallel Builds
- **Matrix Strategy** - Parallel platform builds
- **Make Parallel** - Multi-core compilation
- **Cargo Parallel** - Rust parallel builds

## Monitoring and Analytics

### Build Metrics
- **Build Time** - Platform-specific build duration
- **Success Rate** - Build success/failure rates
- **Artifact Size** - Package size optimization
- **Test Coverage** - Code coverage metrics

### Quality Assurance
- **Automated Testing** - Unit and integration tests
- **Security Scanning** - Vulnerability detection
- **Code Analysis** - Static code analysis
- **Performance Testing** - Application performance

## Future Enhancements

### Planned Features
- **Container Builds** - Docker-based builds
- **Cloud Builds** - Cloud-based compilation
- **Incremental Builds** - Faster rebuild times
- **Cross-Compilation** - Build for multiple targets

### Tooling Improvements
- **Build Dashboard** - Real-time build monitoring
- **Automated Testing** - Expanded test coverage
- **Dependency Updates** - Automated dependency management
- **Security Enhancements** - Advanced security scanning

## Support and Maintenance

### Documentation
- **Build Guides** - Platform-specific instructions
- **API Documentation** - Client API references
- **Troubleshooting** - Common issues and solutions
- **Best Practices** - Development guidelines

### Community
- **GitHub Issues** - Bug tracking and feature requests
- **Discussions** - Community support and collaboration
- **Contributing** - Guidelines for contributions
- **Releases** - Version management and changelog

---

**This comprehensive build system ensures reliable, automated, and secure deployment of all Health OS client applications across all supported platforms.**
