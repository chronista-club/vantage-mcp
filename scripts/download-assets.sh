#!/bin/bash

# Alpine.jsとTabler Iconsをダウンロード

echo "Downloading Alpine.js..."
curl -L https://cdn.jsdelivr.net/npm/alpinejs@3.13.3/dist/cdn.min.js -o static/vendor/libs/alpine.min.js

echo "Downloading Tabler Icons CSS..."
curl -L https://cdn.jsdelivr.net/npm/@tabler/icons@3.34.0/tabler-icons.min.css -o static/vendor/libs/tabler-icons.min.css

echo "Done!"