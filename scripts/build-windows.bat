@echo off
REM Health OS Multi-Platform Build Script for Windows
REM This script builds all client applications for Windows platform

setlocal enabledelayedexpansion

REM Configuration
set BUILD_DIR=build
set DIST_DIR=dist
set TIMESTAMP=%date:~10,4%%date:~4,2%%date:~7,2%_%time:~0,2%%time:~3,2%%time:~6,2%
set TIMESTAMP=%TIMESTAMP: =0%
set VERSION=v1.0.0

REM Create directories
if not exist "%BUILD_DIR%" mkdir "%BUILD_DIR%"
if not exist "%DIST_DIR%" mkdir "%DIST_DIR%"

REM Logging functions
:log_info
echo [INFO] %~1
goto :eof

:log_success
echo [SUCCESS] %~1
goto :eof

:log_warning
echo [WARNING] %~1
goto :eof

:log_error
echo [ERROR] %~1
goto :eof

REM Function to check if command exists
:command_exists
where %1 >nul 2>&1
goto :eof

REM Function to build Android client
:build_android
call :log_info "Building Android client..."

if not exist "java" (
    call :log_error "Java is not installed or not in PATH"
    goto :eof
)

if not exist "%ANDROID_HOME%" (
    call :log_error "ANDROID_HOME is not set"
    call :log_info "Please install Android SDK and set ANDROID_HOME environment variable"
    goto :eof
)

cd clients\android

REM Build APK
call gradlew assembleRelease

if %ERRORLEVEL% neq 0 (
    call :log_error "Android build failed"
    cd ..\..
    goto :eof
)

REM Copy APK to dist directory
copy app\build\outputs\apk\release\app-release.apk "..\%DIST_DIR%\health-os-android-%VERSION%.apk"

cd ..\..
call :log_success "Android client built successfully"
goto :eof

REM Function to build Rust desktop client
:build_desktop_rust
call :log_info "Building Rust desktop client..."

if not exist "cargo" (
    call :log_error "Rust/Cargo is not installed or not in PATH"
    goto :eof
)

cd clients\desktop-rust

REM Install Tauri CLI if not present
call :command_exists "cargo-tauri"
if %ERRORLEVEL% neq 0 (
    call :log_info "Installing Tauri CLI..."
    cargo install tauri-cli
)

REM Build for Windows
cargo tauri build

if %ERRORLEVEL% neq 0 (
    call :log_error "Rust desktop build failed"
    cd ..\..
    goto :eof
)

REM Copy MSI installer
copy src-tauri\target\release\bundle\msi\*.msi "..\%DIST_DIR%"

cd ..\..
call :log_success "Rust desktop client built successfully"
goto :eof

REM Function to build C++ desktop client
:build_desktop_cpp
call :log_info "Building C++ desktop client..."

if not exist "cmake" (
    call :log_error "CMake is not installed or not in PATH"
    goto :eof
)

cd clients\desktop-cpp

REM Create build directory
if not exist "build" mkdir build
cd build

REM Configure and build
cmake .. -G "Visual Studio 17 2022" -A x64
if %ERRORLEVEL% neq 0 (
    call :log_error "CMake configuration failed"
    cd ..\..
    goto :eof
)

cmake --build . --config Release
if %ERRORLEVEL% neq 0 (
    call :log_error "C++ build failed"
    cd ..\..
    goto :eof
)

REM Copy executable
copy Release\health-os-hms.exe "..\..\%DIST_DIR%\health-os-desktop-cpp-windows-%VERSION%.exe"

cd ..\..
call :log_success "C++ desktop client built successfully"
goto :eof

REM Function to build C# desktop client
:build_desktop_csharp
call :log_info "Building C# desktop client..."

if not exist "dotnet" (
    call :log_error ".NET SDK is not installed or not in PATH"
    goto :eof
)

cd clients\desktop-csharp

REM Restore dependencies
dotnet restore
if %ERRORLEVEL% neq 0 (
    call :log_error "C# dependency restore failed"
    cd ..\..
    goto :eof
)

REM Build
dotnet build --configuration Release
if %ERRORLEVEL% neq 0 (
    call :log_error "C# build failed"
    cd ..\..
    goto :eof
)

REM Publish
dotnet publish --configuration Release --output "..\%DIST_DIR%\health-os-desktop-csharp-%VERSION%"

cd ..\..
call :log_success "C# desktop client built successfully"
goto :eof

REM Function to build Python desktop client
:build_desktop_python
call :log_info "Building Python desktop client..."

if not exist "python" (
    call :log_error "Python is not installed or not in PATH"
    goto :eof
)

cd clients\desktop-python

REM Install dependencies
python -m pip install -r requirements.txt
if %ERRORLEVEL% neq 0 (
    call :log_error "Python dependency installation failed"
    cd ..\..
    goto :eof
)

REM Install PyInstaller
python -m pip install pyinstaller

REM Build executable
pyinstaller --onefile --windowed main.py

if %ERRORLEVEL% neq 0 (
    call :log_error "Python build failed"
    cd ..\..
    goto :eof
)

REM Copy executable
copy dist\main.exe "..\%DIST_DIR%\health-os-desktop-python-windows-%VERSION%.exe"

cd ..\..
call :log_success "Python desktop client built successfully"
goto :eof

REM Function to build Web SPA
:build_web_spa
call :log_info "Building Web SPA..."

if not exist "node" (
    call :log_error "Node.js is not installed or not in PATH"
    goto :eof
)

cd clients\web-spa

REM Install dependencies
npm ci
if %ERRORLEVEL% neq 0 (
    call :log_error "Web SPA dependency installation failed"
    cd ..\..
    goto :eof
)

REM Build
npm run build
if %ERRORLEVEL% neq 0 (
    call :log_error "Web SPA build failed"
    cd ..\..
    goto :eof
)

REM Copy build output
xcopy /E /I dist "..\%DIST_DIR%\health-os-web-spa-%VERSION%"

cd ..\..
call :log_success "Web SPA built successfully"
goto :eof

REM Function to create release package
:create_release_package
call :log_info "Creating release package..."

cd %DIST_DIR%

REM Create checksums
certutil -hashfile SHA256 *.* > checksums.txt

cd ..
call :log_success "Release package created successfully"
goto :eof

REM Function to clean up
:cleanup
call :log_info "Cleaning up..."
if exist "%BUILD_DIR%" rmdir /S /Q "%BUILD_DIR%"
call :log_success "Cleanup completed"
goto :eof

REM Main build function
:main
call :log_info "Starting Health OS Windows build..."
call :log_info "Version: %VERSION%"
call :log_info "Timestamp: %TIMESTAMP%"

REM Build all components
call :build_android
call :build_desktop_rust
call :build_desktop_cpp
call :build_desktop_csharp
call :build_desktop_python
call :build_web_spa

REM Create release package
call :create_release_package

REM Clean up
call :cleanup

call :log_success "Windows build completed successfully!"
call :log_info "All artifacts are available in: %DIST_DIR%"
goto :eof

REM Handle script arguments
if "%1"=="android" (
    call :build_android
) else if "%1"=="desktop-rust" (
    call :build_desktop_rust
) else if "%1"=="desktop-cpp" (
    call :build_desktop_cpp
) else if "%1"=="desktop-csharp" (
    call :build_desktop_csharp
) else if "%1"=="desktop-python" (
    call :build_desktop_python
) else if "%1"=="web-spa" (
    call :build_web_spa
) else if "%1"=="all" (
    call :main
) else if "%1"=="clean" (
    call :cleanup
) else (
    echo Usage: %0 [android^|desktop-rust^|desktop-cpp^|desktop-csharp^|desktop-python^|web-spa^|all^|clean]
    exit /b 1
)

endlocal
