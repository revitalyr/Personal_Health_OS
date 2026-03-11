#!/bin/bash

# Health OS Development Environment Setup Script
# This script sets up the complete development environment

set -e

echo "🏥 Setting up Health OS Development Environment..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check if Docker is installed
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    # Check if Docker Compose is installed
    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi
    
    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        print_error "Rust is not installed. Please install Rust first."
        exit 1
    fi
    
    print_status "All prerequisites are installed! ✅"
}

# Setup environment variables
setup_environment() {
    print_status "Setting up environment variables..."
    
    # Create .env file if it doesn't exist
    if [ ! -f .env ]; then
        cat > .env << EOF
# Database Configuration
DATABASE_URL=postgres://postgres:postgres@localhost:5432/health_os
DB_HOST=localhost
DB_PORT=5432
DB_NAME=health_os
DB_USER=postgres
DB_PASSWORD=postgres
DB_MAX_CONNECTIONS=10

# API Configuration
PORT=8080
JWT_SECRET=your-super-secret-jwt-key-change-in-production
CORS_ORIGINS=http://localhost:3000,http://localhost:3001

# Service URLs
TIMELINE_SERVICE_URL=http://localhost:8081
DOCUMENT_SERVICE_URL=http://localhost:8082
AI_SERVICE_URL=http://localhost:8083
DOCTOR_ACCESS_URL=http://localhost:8084

# AI Configuration (optional)
# OPENAI_API_KEY=sk-...
# OLLAMA_URL=http://localhost:11434

# Storage Configuration
STORAGE_TYPE=local
STORAGE_PATH=./storage

# Logging
RUST_LOG=info
EOF
        print_status "Created .env file with default values"
    else
        print_warning ".env file already exists, skipping creation"
    fi
}

# Start infrastructure services
start_infrastructure() {
    print_status "Starting infrastructure services..."
    
    # Start PostgreSQL, Redis, and NATS
    docker-compose -f infrastructure/docker/docker-compose.yml up -d postgres redis nats
    
    # Wait for services to be ready
    print_status "Waiting for services to be ready..."
    sleep 10
    
    # Check if PostgreSQL is ready
    until docker-compose -f infrastructure/docker/docker-compose.yml exec postgres pg_isready -U postgres; do
        print_status "Waiting for PostgreSQL..."
        sleep 2
    done
    
    print_status "Infrastructure services are ready! ✅"
}

# Run database migrations
run_migrations() {
    print_status "Running database migrations..."
    
    # Install sqlx-cli if not installed
    if ! command -v sqlx &> /dev/null; then
        print_status "Installing sqlx-cli..."
        cargo install sqlx-cli --no-default-features --features postgres
    fi
    
    # Run migrations
    DATABASE_URL="postgres://postgres:postgres@localhost:5432/health_os" sqlx migrate run --database-url "postgres://postgres:postgres@localhost:5432/health_os"
    
    print_status "Database migrations completed! ✅"
}

# Build all services
build_services() {
    print_status "Building all services..."
    
    # Build all workspace members
    cargo build --release
    
    print_status "All services built successfully! ✅"
}

# Start all services
start_services() {
    print_status "Starting all Health OS services..."
    
    # Start services in background
    cargo run --package api-gateway &
    API_GATEWAY_PID=$!
    
    cargo run --package timeline-service &
    TIMELINE_PID=$!
    
    cargo run --package document-processor &
    DOCUMENT_PID=$!
    
    cargo run --package ai-report-service &
    AI_PID=$!
    
    cargo run --package doctor-access-service &
    DOCTOR_ACCESS_PID=$!
    
    # Store PIDs for cleanup
    echo $API_GATEWAY_PID > .api-gateway.pid
    echo $TIMELINE_PID > .timeline-service.pid
    echo $DOCUMENT_PID > .document-processor.pid
    echo $AI_PID > .ai-report-service.pid
    echo $DOCTOR_ACCESS_PID > .doctor-access-service.pid
    
    print_status "All services started! ✅"
    print_status "API Gateway: http://localhost:8080"
    print_status "Timeline Service: http://localhost:8081"
    print_status "Document Processor: http://localhost:8082"
    print_status "AI Report Service: http://localhost:8083"
    print_status "Doctor Access Service: http://localhost:8084"
}

# Health check
health_check() {
    print_status "Performing health check..."
    
    # Wait a moment for services to start
    sleep 5
    
    # Check API Gateway
    if curl -f http://localhost:8080/health > /dev/null 2>&1; then
        print_status "API Gateway: Healthy ✅"
    else
        print_error "API Gateway: Unhealthy ❌"
    fi
    
    # Check Timeline Service
    if curl -f http://localhost:8081/health > /dev/null 2>&1; then
        print_status "Timeline Service: Healthy ✅"
    else
        print_error "Timeline Service: Unhealthy ❌"
    fi
    
    # Check Document Processor
    if curl -f http://localhost:8082/health > /dev/null 2>&1; then
        print_status "Document Processor: Healthy ✅"
    else
        print_error "Document Processor: Unhealthy ❌"
    fi
    
    # Check AI Report Service
    if curl -f http://localhost:8083/health > /dev/null 2>&1; then
        print_status "AI Report Service: Healthy ✅"
    else
        print_error "AI Report Service: Unhealthy ❌"
    fi
    
    # Check Doctor Access Service
    if curl -f http://localhost:8084/health > /dev/null 2>&1; then
        print_status "Doctor Access Service: Healthy ✅"
    else
        print_error "Doctor Access Service: Unhealthy ❌"
    fi
}

# Show usage information
show_usage() {
    echo ""
    print_status "Health OS Development Environment is ready!"
    echo ""
    echo "🚀 Quick Start:"
    echo "  # Test the API Gateway"
    echo "  curl http://localhost:8080/health"
    echo ""
    echo "  # Create a test user"
    echo "  curl -X POST http://localhost:8080/auth/login \\"
    echo "    -H 'Content-Type: application/json' \\"
    echo "    -d '{\"email\":\"test@example.com\",\"password\":\"password\"}'"
    echo ""
    echo "  # View logs"
    echo "  docker-compose -f infrastructure/docker/docker-compose.yml logs -f"
    echo ""
    echo "  # Stop all services"
    echo "  ./scripts/stop.sh"
    echo ""
    echo "📚 Documentation:"
    echo "  - API Documentation: http://localhost:8080/docs"
    echo "  - Architecture: ./ARCHITECTURE.md"
    echo "  - README: ./README.md"
    echo ""
}

# Cleanup function
cleanup() {
    print_status "Cleaning up..."
    
    # Kill running services
    if [ -f .api-gateway.pid ]; then
        kill $(cat .api-gateway.pid) 2>/dev/null || true
        rm .api-gateway.pid
    fi
    
    if [ -f .timeline-service.pid ]; then
        kill $(cat .timeline-service.pid) 2>/dev/null || true
        rm .timeline-service.pid
    fi
    
    if [ -f .document-processor.pid ]; then
        kill $(cat .document-processor.pid) 2>/dev/null || true
        rm .document-processor.pid
    fi
    
    if [ -f .ai-report-service.pid ]; then
        kill $(cat .ai-report-service.pid) 2>/dev/null || true
        rm .ai-report-service.pid
    fi
    
    if [ -f .doctor-access-service.pid ]; then
        kill $(cat .doctor-access-service.pid) 2>/dev/null || true
        rm .doctor-access-service.pid
    fi
    
    print_status "Cleanup completed!"
}

# Set up signal handlers
trap cleanup EXIT INT TERM

# Main execution
main() {
    echo "🏥 Health OS Development Environment Setup"
    echo "=========================================="
    
    check_prerequisites
    setup_environment
    start_infrastructure
    run_migrations
    build_services
    start_services
    health_check
    show_usage
    
    print_status "Development environment is ready! 🎉"
    
    # Keep the script running to maintain services
    print_status "Press Ctrl+C to stop all services..."
    wait
}

# Run main function
main "$@"
