#!/bin/bash

# setup_podman.sh - Configures Agency to use Podman instead of Docker
# This script finds the active Podman socket and exports it as DOCKER_HOST.

# 1. Get the socket path from the default Podman machine
SOCKET_PATH=$(podman machine inspect podman-machine-default --format '{{.ConnectionInfo.PodmanSocket.Path}}')

if [ -z "$SOCKET_PATH" ]; then
    echo "❌ Podman machine not found or not running."
    echo "Run: podman machine start"
    exit 1
fi

export DOCKER_HOST="unix://$SOCKET_PATH"

echo "✅ Podman integration active."
echo "📍 DOCKER_HOST=$DOCKER_HOST"

# Print instructions for the user
echo ""
echo "To use this in your current shell, run:"
echo "  source scripts/setup_podman.sh"
