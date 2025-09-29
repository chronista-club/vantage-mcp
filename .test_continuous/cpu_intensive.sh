#!/bin/bash
echo "[$(date +%H:%M:%S)] CPU intensive process started (PID $$)"
trap 'echo "[$(date +%H:%M:%S)] Terminated"; exit 0' SIGTERM
while true; do
    # CPU負荷をシミュレート（実際には軽い処理）
    for i in {1..1000}; do
        echo $((i * i)) > /dev/null
    done
    sleep 1
done
