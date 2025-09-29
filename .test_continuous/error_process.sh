#!/bin/bash
echo "[$(date +%H:%M:%S)] Error process started (PID $$)"
sleep 2
echo "[$(date +%H:%M:%S)] Simulating error..."
exit 1
