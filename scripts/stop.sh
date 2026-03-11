#!/bin/bash

# Health OS Development Environment Stop Script
# This script stops all running services and cleans up

set -e

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

# Stop running services
stop_services() {
    print_status "Stopping Health OS services..."
    
    # Kill running services by PID files
    if [ -f .api-gateway.pid ]; then
        PID=$(cat .api-gateway.pid)
        if kill -0 $PID 2>/dev/null; then
            print_status "Stopping API Gateway (PID: $PID)..."
            kill $PID
        fi
        rm .api-gateway.pid
    fi
    
    if [ -f .timeline-service.pid ]; then
        PID=$(cat .timeline-service.pid)
        if kill -0 $PID 2>/dev/null; then
            print_status "Stopping Timeline Service (PID: $PID)..."
            kill $PID
        fi
        rm .timeline-service.pid
    fi
    
    if [ -f .document-processor.pid ]; then
        PID=$(cat .document-processor.pid)
        if kill -0 $PID 2>/dev/null; then
            print_status "Stopping Document Processor (PID: $PID)..."
            kill $PID
        fi
        rm .document-processor.pid
    fi
    
    if [ -f .ai-report-service.pid ]; then
        PID=$(cat .ai-report-service.pid)
        if kill -0 $PID 2>/dev/null; then
            print_status "Stopping AI Report Service (PID: $PID)..."
            kill $PID
        fi
        rm .ai-report-service.pid
    fi
    
    if [ -f .doctor-access-service.pid ]; then
        PID=$(cat .doctor-access-service.pid)
        if kill -0 $PID 2>/dev/null; then
            print_status "Stopping Doctor Access Service (PID: $PID)..."
            kill $PID
        fi
        rm .doctor-access-service.pid
    fi
    
    # Force kill any remaining processes
    pkill -f "api-gateway" 2>/dev/null || true
    pkill -f "timeline-service" 2>/dev/null || true
    pkill -f "document-processor" 2>/dev/null || true
    pkill -f "ai-report-service" 2>/dev/null || true
    pkill -f "doctor-access-service" 2>/dev/null || true
    
    print_status "All services stopped! ✅"
}

# Stop infrastructure services
stop_infrastructure() {
    print_status "Stopping infrastructure services..."
    
    # Stop Docker containers
    if command -v docker-compose &> /dev/null; then
        docker-compose -f infrastructure/docker/docker-compose.yml down
    fi
    
    print_status "Infrastructure services stopped! ✅"
}

# Clean up temporary files
cleanup() {
    print_status "Cleaning up temporary files..."
    
    # Remove PID files
    rm -f .*.pid
    
    # Remove temporary storage
    rm -rf ./storage
    
    # Remove log files
    rm -f *.log
    
    print_status "Cleanup completed! ✅"
}

# Show status
show_status() {
    print_status "Checking service status..."
    
    # Check if any services are still running
    if pgrep -f "api-gateway" > /dev/null; then
        print_warning "API Gateway is still running"
    fi
    
    if pgrep -f "timeline-service" > /dev/null; then
        print_warning "Timeline Service is still running"
    fi
    
    if pgrep -f "document-processor" > /dev/null; then
        print_warning "Document Processor is still running"
    fi
    
    if pgrep -f "ai-report-service" > /dev/null; then
        print_warning "AI Report Service is still running"
    fi
    
    if pgrep -f "doctor-access-service" > /dev/null; then
        print_warning "Doctor Access Service is still running"
    fi
    
    # Check Docker containers
    if command -v docker-compose &> /dev/null; then
        if docker-compose -f infrastructure/docker/docker-compose.yml ps | grep -q "Up"; then
            print_warning "Some Docker containers are still running"
        else
            print_status "All Docker containers are stopped"
        fi
    fi
}

# Main execution
main() {
    echo "🛑 Stopping Health OS Development Environment"
    echo "============================================="
    
    stop_services
    stop_infrastructure
    cleanup
    show_status
    
    print_status "Health OS Development Environment stopped! 🎉"
}

# Run main function
main "$@"
