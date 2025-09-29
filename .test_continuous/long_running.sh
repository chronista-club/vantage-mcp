#!/bin/bash
echo "[$(date +%H:%M:%S)] Long running process started (PID $$)"
trap 'echo "[$(date +%H:%M:%S)] Received SIGTERM, shutting down..."; exit 0' SIGTERM
count=0
while true; do
    echo "[$(date +%H:%M:%S)] Still running... (count: $count)"
    ((count++))
    sleep 5
done
