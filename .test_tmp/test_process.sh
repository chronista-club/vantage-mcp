#!/bin/bash
echo "[$(date +%H:%M:%S)] Process $1 started with PID $$"
trap 'echo "[$(date +%H:%M:%S)] Process $1 received SIGTERM, shutting down gracefully..."; exit 0' SIGTERM
trap 'echo "[$(date +%H:%M:%S)] Process $1 received SIGINT"; exit 0' SIGINT
while true; do
    echo "[$(date +%H:%M:%S)] Process $1 running..."
    sleep 2
done
