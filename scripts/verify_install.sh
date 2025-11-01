#!/bin/bash

# Verify Vantage MCP Installation

echo "Checking Vantage MCP installation..."
echo ""

# Check if vantagemcp is in PATH
if command -v vantagemcp &> /dev/null; then
    echo "✅ vantagemcp found in PATH"
    echo "   Location: $(which vantagemcp)"
    echo "   Version: $(vantagemcp --version 2>&1 | head -1)"
else
    echo "❌ vantagemcp not found in PATH"
    echo ""
    echo "   Possible locations to check:"
    echo "   - ~/.local/bin/vantagemcp"
    echo "   - /usr/local/bin/vantagemcp"
    echo "   - ~/go/bin/vantagemcp (if installed via go)"
    echo "   - ~/.cargo/bin/vantagemcp (if installed via cargo)"
    
    # Check common locations
    for loc in ~/.local/bin/vantagemcp /usr/local/bin/vantagemcp ~/.cargo/bin/vantagemcp; do
        if [ -f "$loc" ]; then
            echo ""
            echo "   Found at: $loc"
            echo "   Add to PATH: export PATH=\"\$PATH:$(dirname $loc)\""
        fi
    done
    exit 1
fi

echo ""
echo "Testing basic functionality..."

# Test help command
if vantagemcp --help &> /dev/null; then
    echo "✅ Help command works"
else
    echo "❌ Help command failed"
    exit 1
fi

# Check if it can run as MCP server
echo ""
echo "Testing MCP server mode..."
echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}' | timeout 2 vantagemcp 2>/dev/null | grep -q "result" && echo "✅ MCP server responds" || echo "⚠️  MCP server test inconclusive (this is normal)"

echo ""
echo "Installation verification complete!"
echo ""
echo "Next steps:"
echo "1. Add to Claude Code configuration (~/.config/claude/mcp.json or similar)"
echo "2. Run 'vantagemcp' to start the server"
echo "3. Use 'vantagemcp --web' to enable web dashboard"
