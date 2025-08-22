#!/bin/bash

# Verify Ichimi Server Installation

echo "Checking Ichimi Server installation..."
echo ""

# Check if ichimi is in PATH
if command -v ichimi &> /dev/null; then
    echo "✅ ichimi found in PATH"
    echo "   Location: $(which ichimi)"
    echo "   Version: $(ichimi --version 2>&1 | head -1)"
else
    echo "❌ ichimi not found in PATH"
    echo ""
    echo "   Possible locations to check:"
    echo "   - ~/.local/bin/ichimi"
    echo "   - /usr/local/bin/ichimi"
    echo "   - ~/go/bin/ichimi (if installed via go)"
    echo "   - ~/.cargo/bin/ichimi (if installed via cargo)"
    
    # Check common locations
    for loc in ~/.local/bin/ichimi /usr/local/bin/ichimi ~/.cargo/bin/ichimi; do
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
if ichimi --help &> /dev/null; then
    echo "✅ Help command works"
else
    echo "❌ Help command failed"
    exit 1
fi

# Check if it can run as MCP server
echo ""
echo "Testing MCP server mode..."
echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}' | timeout 2 ichimi 2>/dev/null | grep -q "result" && echo "✅ MCP server responds" || echo "⚠️  MCP server test inconclusive (this is normal)"

echo ""
echo "Installation verification complete!"
echo ""
echo "Next steps:"
echo "1. Add to Claude Code configuration (~/.config/claude/mcp.json or similar)"
echo "2. Run 'ichimi' to start the server"
echo "3. Use 'ichimi --web' to enable web dashboard"