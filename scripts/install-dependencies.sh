#!/bin/bash

# Health OS Multi-Platform Dependency Installer
# This script installs all required dependencies for building all client applications

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

# Function to detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)     echo "linux";;
        Darwin*)    echo "macos";;
        CYGWIN*|MINGW*|MSYS*) echo "windows";;
        *)          echo "unknown";;
    esac
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install package based on package manager
install_package() {
    local package="$1"
    local os=$(detect_os)
    
    case $os in
        "linux")
            if command_exists "apt-get"; then
                sudo apt-get update && sudo apt-get install -y "$package"
            elif command_exists "yum"; then
                sudo yum install -y "$package"
            elif command_exists "dnf"; then
                sudo dnf install -y "$package"
            elif command_exists "pacman"; then
                sudo pacman -S --noconfirm "$package"
            else
                log_error "Unsupported Linux package manager"
                return 1
            fi
            ;;
        "macos")
            if command_exists "brew"; then
                brew install "$package"
            else
                log_error "Homebrew not found. Please install Homebrew first."
                return 1
            fi
            ;;
        "windows")
            if command_exists "choco"; then
                choco install "$package"
            elif command_exists "winget"; then
                winget install "$package"
            else
                log_error "Chocolatey or Winget not found. Please install one first."
                return 1
            fi
            ;;
        *)
            log_error "Unsupported operating system"
            return 1
            ;;
    esac
}

# Function to install Rust
install_rust() {
    log_info "Installing Rust..."
    
    if command_exists "cargo"; then
        log_success "Rust is already installed"
        cargo update
    else
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    
    # Install Tauri CLI
    if ! command_exists "cargo-tauri"; then
        log_info "Installing Tauri CLI..."
        cargo install tauri-cli
    fi
    
    log_success "Rust installation completed"
}

# Function to install Node.js
install_nodejs() {
    log_info "Installing Node.js..."
    
    if command_exists "node"; then
        log_success "Node.js is already installed"
        node --version
    else
        case $(detect_os) in
            "linux")
                # Install Node.js using NodeSource
                curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
                sudo apt-get install -y nodejs
                ;;
            "macos")
                install_package "node"
                ;;
            "windows")
                install_package "nodejs"
                ;;
        esac
    fi
    
    # Install global npm packages
    log_info "Installing global npm packages..."
    npm install -g @vue/cli
    npm install -g vite
    
    log_success "Node.js installation completed"
}

# Function to install Java and Android SDK
install_android() {
    log_info "Installing Android development tools..."
    
    # Install Java
    if command_exists "java"; then
        log_success "Java is already installed"
        java -version
    else
        case $(detect_os) in
            "linux")
                install_package "openjdk-17-jdk"
                ;;
            "macos")
                install_package "openjdk@17"
                ;;
            "windows")
                install_package "openjdk"
                ;;
        esac
    fi
    
    # Install Android SDK
    if [ -z "$ANDROID_HOME" ]; then
        log_info "Installing Android SDK..."
        
        case $(detect_os) in
            "linux")
                install_package "android-sdk"
                install_package "android-sdk-platform-tools"
                ;;
            "macos")
                install_package "android-sdk"
                install_package "android-platform-tools"
                ;;
            "windows")
                install_package "android-sdk"
                install_package "android-platform-tools"
                ;;
        esac
        
        # Set ANDROID_HOME
        echo "export ANDROID_HOME=\$HOME/Android/Sdk" >> ~/.bashrc
        echo "export PATH=\$PATH:\$ANDROID_HOME/tools:\$ANDROID_HOME/platform-tools" >> ~/.bashrc
        export ANDROID_HOME="$HOME/Android/Sdk"
        export PATH="$PATH:$ANDROID_HOME/tools:$ANDROID_HOME/platform-tools"
    else
        log_success "Android SDK is already installed"
    fi
    
    log_success "Android development tools installation completed"
}

# Function to install iOS development tools (macOS only)
install_ios() {
    if [ "$(detect_os)" != "macos" ]; then
        log_warning "iOS development requires macOS, skipping..."
        return 0
    fi
    
    log_info "Installing iOS development tools..."
    
    # Install Xcode Command Line Tools
    if command_exists "xcodebuild"; then
        log_success "Xcode is already installed"
        xcodebuild -version
    else
        xcode-select --install
    fi
    
    log_success "iOS development tools installation completed"
}

# Function to install C++ development tools
install_cpp() {
    log_info "Installing C++ development tools..."
    
    case $(detect_os) in
        "linux")
            install_package "build-essential"
            install_package "cmake"
            install_package "qt6-base-dev"
            install_package "qt6-tools-dev"
            install_package "libqt6webengine6-dev"
            ;;
        "macos")
            install_package "cmake"
            install_package "qt@6"
            # Link Qt6
            if [ -d "/opt/homebrew/opt/qt@6" ]; then
                echo 'export PATH="/opt/homebrew/opt/qt@6/bin:$PATH"' >> ~/.bashrc
                export PATH="/opt/homebrew/opt/qt@6/bin:$PATH"
            fi
            ;;
        "windows")
            install_package "visualstudio2022buildtools"
            install_package "cmake"
            install_package "qt6"
            ;;
    esac
    
    log_success "C++ development tools installation completed"
}

# Function to install .NET development tools
install_dotnet() {
    log_info "Installing .NET development tools..."
    
    if command_exists "dotnet"; then
        log_success ".NET is already installed"
        dotnet --version
    else
        case $(detect_os) in
            "linux")
                # Install .NET SDK
                curl -sSL https://dot.net/v1/dotnet-install.sh | bash /dev/stdin --channel 7.0
                ;;
            "macos")
                install_package "dotnet-sdk"
                ;;
            "windows")
                install_package "dotnet-sdk-7.0"
                ;;
        esac
    fi
    
    log_success ".NET development tools installation completed"
}

# Function to install Python development tools
install_python() {
    log_info "Installing Python development tools..."
    
    if command_exists "python3"; then
        log_success "Python 3 is already installed"
        python3 --version
    else
        case $(detect_os) in
            "linux")
                install_package "python3"
                install_package "python3-pip"
                install_package "python3-tk"
                install_package "python3-dev"
                ;;
            "macos")
                install_package "python@3.11"
                ;;
            "windows")
                install_package "python"
                ;;
        esac
    fi
    
    # Install Python packages
    log_info "Installing Python packages..."
    pip3 install --user -r clients/desktop-python/requirements.txt
    
    log_success "Python development tools installation completed"
}

# Function to install system dependencies
install_system_deps() {
    log_info "Installing system dependencies..."
    
    case $(detect_os) in
        "linux")
            install_package "curl"
            install_package "wget"
            install_package "git"
            install_package "build-essential"
            install_package "pkg-config"
            install_package "libssl-dev"
            install_package "libwebkit2gtk-4.0-dev"
            install_package "libgtk-3-dev"
            install_package "libayatana-appindicator3-dev"
            install_package "librsvg2-dev"
            ;;
        "macos")
            install_package "curl"
            install_package "wget"
            install_package "git"
            install_package "pkg-config"
            install_package "openssl"
            ;;
        "windows")
            install_package "git"
            install_package "curl"
            install_package "wget"
            ;;
    esac
    
    log_success "System dependencies installation completed"
}

# Function to verify installations
verify_installations() {
    log_info "Verifying installations..."
    
    local tools=("git" "curl" "wget" "node" "npm" "python3" "pip3" "cargo" "cmake")
    local missing_tools=()
    
    for tool in "${tools[@]}"; do
        if command_exists "$tool"; then
            log_success "$tool is installed"
        else
            log_warning "$tool is not installed"
            missing_tools+=("$tool")
        fi
    done
    
    # Platform-specific checks
    case $(detect_os) in
        "linux")
            if command_exists "java"; then
                log_success "Java is installed"
            else
                missing_tools+=("java")
            fi
            ;;
        "macos")
            if command_exists "xcodebuild"; then
                log_success "Xcode is installed"
            else
                missing_tools+=("xcodebuild")
            fi
            ;;
        "windows")
            if command_exists "dotnet"; then
                log_success ".NET is installed"
            else
                missing_tools+=("dotnet")
            fi
            ;;
    esac
    
    if [ ${#missing_tools[@]} -eq 0 ]; then
        log_success "All required tools are installed!"
    else
        log_warning "Missing tools: ${missing_tools[*]}"
        log_info "You may need to install these manually"
    fi
}

# Main installation function
main() {
    log_info "Starting Health OS multi-platform dependency installation..."
    log_info "Detected OS: $(detect_os)"
    
    # Install system dependencies first
    install_system_deps
    
    # Install language-specific tools
    install_rust
    install_nodejs
    install_python
    install_cpp
    install_dotnet
    
    # Install platform-specific tools
    install_android
    install_ios
    
    # Verify installations
    verify_installations
    
    log_success "Dependency installation completed!"
    log_info "You can now build all platforms using: ./scripts/build-all-platforms.sh all"
}

# Handle script arguments
case "${1:-all}" in
    "rust")
        install_rust
        ;;
    "nodejs")
        install_nodejs
        ;;
    "android")
        install_android
        ;;
    "ios")
        install_ios
        ;;
    "cpp")
        install_cpp
        ;;
    "dotnet")
        install_dotnet
        ;;
    "python")
        install_python
        ;;
    "system")
        install_system_deps
        ;;
    "verify")
        verify_installations
        ;;
    "all")
        main
        ;;
    *)
        echo "Usage: $0 [rust|nodejs|android|ios|cpp|dotnet|python|system|verify|all]"
        exit 1
        ;;
esac
