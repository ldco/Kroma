#!/bin/bash
# Start Kroma Frontend with proper memory settings
#
# This script starts the Nuxt development server with:
# - Path resolution relative to script location
# - Directory validation
# - Memory optimization settings
#
# Usage: ./start-frontend.sh

set -euo pipefail

# Derive path relative to script location
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Validate required directories
if [[ ! -d "${SCRIPT_DIR}" ]]; then
    echo "ERROR: Script directory not found: ${SCRIPT_DIR}"
    exit 1
fi

cd "${SCRIPT_DIR}"

# Validate package.json exists
if [[ ! -f "package.json" ]]; then
    echo "ERROR: package.json not found in ${SCRIPT_DIR}"
    echo "Ensure you are running this script from the front-end-puppet-master directory."
    exit 1
fi

# Check dependencies
if [[ ! -d "node_modules" ]]; then
    echo "Frontend dependencies not found. Installing..."
    npm install
    if [[ $? -ne 0 ]]; then
        echo "ERROR: Frontend dependency installation failed."
        exit 1
    fi
    echo "Frontend dependencies installed successfully."
fi

# Clean cache
rm -rf .nuxt

# Set memory limit to 8GB
export NODE_OPTIONS="--max-old-space-size=8192"

# Start dev server
echo "Starting Nuxt dev server..."
echo "Memory limit: 8GB"
echo "Working directory: ${SCRIPT_DIR}"
echo ""

npm run dev
