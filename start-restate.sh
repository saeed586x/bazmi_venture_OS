#!/bin/bash
# Script to download and run Restate server locally

echo "=== DOWNLOADING AND RUNNING RESTATE SERVER ==="

# Check if restate-server binary exists
if [ ! -f "restate-server" ]; then
    echo "Downloading Restate server..."
    
    # Try to download using curl
    if command -v curl &> /dev/null; then
        # Try to get the latest release (this is a simplified approach)
        curl -L -o restate-server.tar.gz "https://github.com/restatedev/restate/releases/latest/download/restate-server-x86_64-unknown-linux-gnu.tar.gz"
        if [ $? -eq 0 ]; then
            tar -xzf restate-server.tar.gz
            rm restate-server.tar.gz
            chmod +x restate-server
            echo "✅ Restate server downloaded successfully"
        else
            echo "❌ Failed to download Restate server"
            exit 1
        fi
    else
        echo "❌ curl not available, cannot download Restate server"
        exit 1
    fi
else
    echo "✅ Restate server binary already exists"
fi

echo "Starting Restate server..."
./restate-server &
RESTATE_PID=$!

# Wait a moment for server to start
sleep 3

# Check if server is running
if kill -0 $RESTATE_PID 2>/dev/null; then
    echo "✅ Restate server started successfully (PID: $RESTATE_PID)"
    echo "Server running on http://localhost:8080"
    echo "Use 'kill $RESTATE_PID' to stop the server"
else
    echo "❌ Failed to start Restate server"
    exit 1
fi
