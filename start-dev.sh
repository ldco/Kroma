#!/bin/bash
# Start Kroma Development Servers
# 
# This script starts both backend and frontend services with:
# - Dependency validation before startup
# - Health checks for service readiness
# - Coordinated teardown on failure or exit
#
# Usage: ./start-dev.sh

set -euo pipefail

# Derive paths relative to script location
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FRONTEND_DIR="${SCRIPT_DIR}/front-end-puppet-master"
BACKEND_DIR="${SCRIPT_DIR}/src-tauri"

# Configuration
BACKEND_URL="http://127.0.0.1:8788"
FRONTEND_URL="http://localhost:3000"
HEALTH_CHECK_TIMEOUT=60
HEALTH_CHECK_INTERVAL=2

# Process IDs
BACKEND_PID=""
FRONTEND_PID=""

# Cleanup function for coordinated teardown
cleanup() {
    local exit_code=$?
    echo ""
    echo "=== Shutting down development servers ==="
    
    if [[ -n "${BACKEND_PID}" ]] && kill -0 "${BACKEND_PID}" 2>/dev/null; then
        echo "Stopping backend (PID: ${BACKEND_PID})..."
        kill "${BACKEND_PID}" 2>/dev/null || true
        wait "${BACKEND_PID}" 2>/dev/null || true
    fi
    
    if [[ -n "${FRONTEND_PID}" ]] && kill -0 "${FRONTEND_PID}" 2>/dev/null; then
        echo "Stopping frontend (PID: ${FRONTEND_PID})..."
        kill "${FRONTEND_PID}" 2>/dev/null || true
        wait "${FRONTEND_PID}" 2>/dev/null || true
    fi
    
    echo "Cleanup complete."
    exit ${exit_code}
}

# Set trap for coordinated teardown on exit, interrupt, or error
trap cleanup EXIT INT TERM

# Validate required directories exist
validate_directories() {
    echo "=== Validating directory structure ==="
    
    if [[ ! -d "${SCRIPT_DIR}" ]]; then
        echo "ERROR: Script directory not found: ${SCRIPT_DIR}"
        exit 1
    fi
    
    if [[ ! -d "${FRONTEND_DIR}" ]]; then
        echo "ERROR: Frontend directory not found: ${FRONTEND_DIR}"
        exit 1
    fi
    
    if [[ ! -d "${BACKEND_DIR}" ]]; then
        echo "ERROR: Backend directory not found: ${BACKEND_DIR}"
        exit 1
    fi
    
    echo "Directory validation passed."
}

# Check if frontend dependencies are installed
check_frontend_deps() {
    echo "=== Checking frontend dependencies ==="
    
    if [[ ! -d "${FRONTEND_DIR}/node_modules" ]]; then
        echo "Frontend dependencies not found. Installing..."
        cd "${FRONTEND_DIR}"
        npm install
        if [[ $? -ne 0 ]]; then
            echo "ERROR: Frontend dependency installation failed."
            exit 1
        fi
        echo "Frontend dependencies installed successfully."
    else
        echo "Frontend dependencies already installed."
    fi
}

# Check if backend dependencies are installed (Rust)
check_backend_deps() {
    echo "=== Checking backend dependencies ==="
    
    if ! command -v cargo &> /dev/null; then
        echo "ERROR: Rust toolchain (cargo) not found. Please install Rust."
        echo "Visit: https://rustup.rs/"
        exit 1
    fi
    
    echo "Rust toolchain found: $(cargo --version)"
}

# Health check for backend
check_backend_health() {
    local max_attempts=$((HEALTH_CHECK_TIMEOUT / HEALTH_CHECK_INTERVAL))
    local attempt=0

    echo "Waiting for backend to be ready..."

    while [[ ${attempt} -lt ${max_attempts} ]]; do
        if curl -s --connect-timeout 5 --max-time 10 "${BACKEND_URL}/health" > /dev/null 2>&1; then
            echo "Backend is ready."
            return 0
        fi
        attempt=$((attempt + 1))
        sleep ${HEALTH_CHECK_INTERVAL}
    done

    echo "ERROR: Backend failed to become ready within ${HEALTH_CHECK_TIMEOUT} seconds."
    return 1
}

# Health check for frontend
check_frontend_health() {
    local max_attempts=$((HEALTH_CHECK_TIMEOUT / HEALTH_CHECK_INTERVAL))
    local attempt=0

    echo "Waiting for frontend to be ready..."

    while [[ ${attempt} -lt ${max_attempts} ]]; do
        if curl -s --connect-timeout 5 --max-time 10 "${FRONTEND_URL}/app" > /dev/null 2>&1; then
            echo "Frontend is ready."
            return 0
        fi
        attempt=$((attempt + 1))
        sleep ${HEALTH_CHECK_INTERVAL}
    done

    echo "ERROR: Frontend failed to become ready within ${HEALTH_CHECK_TIMEOUT} seconds."
    echo "DIAGNOSTICS:"
    echo "  - Check frontend logs for errors"
    echo "  - Verify frontend process is running: ps aux | grep nuxt"
    echo "  - Check port binding: ss -tlnp | grep 3000"
    echo "  - Try manual curl: curl -v ${FRONTEND_URL}/app"
    return 1  # Strict failure - do not mask readiness issues
}

# Start backend service
start_backend() {
    echo ""
    echo "=== Starting Kroma Backend ==="
    
    cd "${BACKEND_DIR}"
    cargo run &
    BACKEND_PID=$!
    echo "Backend PID: ${BACKEND_PID}"
    
    # Verify process started
    if ! kill -0 "${BACKEND_PID}" 2>/dev/null; then
        echo "ERROR: Backend process failed to start."
        exit 1
    fi
}

# Start frontend service
start_frontend() {
    echo ""
    echo "=== Starting Kroma Frontend ==="

    cd "${FRONTEND_DIR}"

    # Clean build cache to avoid stale state
    rm -rf .nuxt

    # Set memory limit
    export NODE_OPTIONS="--max-old-space-size=8192"

    # Start with --host to bind to all interfaces (not just IPv6 localhost)
    npm run dev -- --host 0.0.0.0 &
    FRONTEND_PID=$!
    echo "Frontend PID: ${FRONTEND_PID}"

    # Verify process started
    if ! kill -0 "${FRONTEND_PID}" 2>/dev/null; then
        echo "ERROR: Frontend process failed to start."
        exit 1
    fi
}

# Main execution
main() {
    echo "=== Kroma Development Server Startup ==="
    echo "Script directory: ${SCRIPT_DIR}"
    echo ""
    
    # Validate environment
    validate_directories
    check_backend_deps
    check_frontend_deps
    
    # Start services
    start_backend
    start_frontend
    
    # Health checks
    echo ""
    echo "=== Running Health Checks ==="
    
    if ! check_backend_health; then
        echo "Backend health check failed."
        exit 1
    fi
    
    if ! check_frontend_health; then
        echo "Frontend health check failed."
        exit 1
    fi
    
    # Success message
    echo ""
    echo "=== Servers Running ==="
    echo "Backend:  ${BACKEND_URL}"
    echo "Frontend: ${FRONTEND_URL}/app"
    echo ""
    echo "Press Ctrl+C to stop all servers"
    echo ""

    # First-exit supervision: monitor both processes and exit when either dies
    supervise_processes
}

# First-exit supervision loop
# Detects when either child process exits and terminates the survivor
supervise_processes() {
    local backend_exited=false
    local frontend_exited=false
    local exit_code=0

    while true; do
        # Check if backend is still running
        if ! kill -0 "${BACKEND_PID}" 2>/dev/null; then
            if [[ "${backend_exited}" == false ]]; then
                echo ""
                echo "=== Backend process exited (PID: ${BACKEND_PID}) ==="
                backend_exited=true
            fi
        fi

        # Check if frontend is still running
        if ! kill -0 "${FRONTEND_PID}" 2>/dev/null; then
            if [[ "${frontend_exited}" == false ]]; then
                echo ""
                echo "=== Frontend process exited (PID: ${FRONTEND_PID}) ==="
                frontend_exited=true
            fi
        fi

        # If either process has exited, terminate the survivor and exit
        if [[ "${backend_exited}" == true ]] || [[ "${frontend_exited}" == true ]]; then
            echo "=== Service failure detected - shutting down surviving process ==="

            # Terminate backend if still running
            if [[ -n "${BACKEND_PID}" ]] && kill -0 "${BACKEND_PID}" 2>/dev/null; then
                echo "Stopping backend (PID: ${BACKEND_PID})..."
                kill "${BACKEND_PID}" 2>/dev/null || true
            fi

            # Terminate frontend if still running
            if [[ -n "${FRONTEND_PID}" ]] && kill -0 "${FRONTEND_PID}" 2>/dev/null; then
                echo "Stopping frontend (PID: ${FRONTEND_PID})..."
                kill "${FRONTEND_PID}" 2>/dev/null || true
            fi

            # Wait for processes to fully terminate
            wait "${BACKEND_PID}" 2>/dev/null || true
            wait "${FRONTEND_PID}" 2>/dev/null || true

            echo "=== All processes terminated ==="
            exit_code=1
            break
        fi

        # Sleep briefly before next check (avoid busy-waiting)
        sleep 1
    done

    exit ${exit_code}
}

# Run main
main
