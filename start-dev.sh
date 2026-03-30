#!/bin/bash
# Start Kroma Development Servers

echo "=== Starting Kroma Backend ==="
cd /run/media/ldco/3734114f-7123-41f5-8f63-7f43c94879eb/CURRENT_WORKING_DEV/Kroma/app
npm run backend:rust &
BACKEND_PID=$!
echo "Backend PID: $BACKEND_PID"

# Wait for backend to start
sleep 5

echo ""
echo "=== Starting Kroma Frontend ==="
cd /run/media/ldco/3734114f-7123-41f5-8f63-7f43c94879eb/CURRENT_WORKING_DEV/Kroma/app/front-end-puppet-master
rm -rf .nuxt
npm run dev &
FRONTEND_PID=$!
echo "Frontend PID: $FRONTEND_PID"

echo ""
echo "=== Servers Starting ==="
echo "Backend:  http://127.0.0.1:8788"
echo "Frontend: http://localhost:3000/app"
echo ""
echo "Press Ctrl+C to stop all servers"
echo ""

# Wait for both processes
wait $BACKEND_PID $FRONTEND_PID
