#!/bin/bash
# Start Kroma Frontend with proper memory settings

cd /run/media/ldco/3734114f-7123-41f5-8f63-7f43c94879eb/CURRENT_WORKING_DEV/Kroma/app/front-end-puppet-master

# Clean cache
rm -rf .nuxt

# Set memory limit to 8GB
export NODE_OPTIONS="--max-old-space-size=8192"

# Start dev server
echo "Starting Nuxt dev server..."
echo "Memory limit: 8GB"
echo ""

npm run dev
