#!/bin/bash
echo "[$(date +%H:%M:%S)] Test process started with PID $$"
trap 'echo "[$(date +%H:%M:%S)] Received SIGTERM, shutting down gracefully..."; exit 0' SIGTERM
while true; do
    echo "[$(date +%H:%M:%S)] Process running..."
    sleep 2
done
