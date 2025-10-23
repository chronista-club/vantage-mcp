#!/bin/bash

# MCP Protocol Test for Vantage Server

echo "=== MCP Protocol Test ==="

cleanup() {
    echo "Cleaning up..."
    pkill -f "npx.*mcp.*test" 2>/dev/null
    pkill -f "vantagemcp" 2>/dev/null
}
trap cleanup EXIT

echo "1. Starting vantage MCP server..."
./target/release/vantagemcp 2>&1 | tee /tmp/vantage_mcp_test.log &
VANTAGE_PID=$!
sleep 2

echo "2. Testing MCP connection..."
# MCPクライアントテスト（簡易版）
cat > /tmp/test_mcp_client.js << 'EOF'
const { Client } = require('@modelcontextprotocol/sdk/client/index.js');
const { StdioClientTransport } = require('@modelcontextprotocol/sdk/client/stdio.js');

async function testMCP() {
    const transport = new StdioClientTransport({
        command: './target/release/vantagemcp',
        args: [],
    });
    
    const client = new Client({
        name: 'test-client',
        version: '1.0.0',
    }, {
        capabilities: {}
    });
    
    await client.connect(transport);
    
    // Test ping
    const pingResult = await client.callTool('ping', {});
    console.log('Ping result:', pingResult);
    
    // Test echo
    const echoResult = await client.callTool('echo', { message: 'Hello MCP!' });
    console.log('Echo result:', echoResult);
    
    // Test create process
    const createResult = await client.callTool('create_process', {
        id: 'mcp-test',
        command: 'echo',
        args: ['MCP Test']
    });
    console.log('Create result:', createResult);
    
    await client.close();
    process.exit(0);
}

testMCP().catch(err => {
    console.error('Test failed:', err);
    process.exit(1);
});
EOF

# MCPパッケージがインストールされているか確認
if [ -d "node_modules/@modelcontextprotocol" ]; then
    echo "3. Running MCP client test..."
    node /tmp/test_mcp_client.js
    
    if [ $? -eq 0 ]; then
        echo "✓ MCP protocol test passed"
    else
        echo "✗ MCP protocol test failed"
        exit 1
    fi
else
    echo "⚠ MCP SDK not installed, skipping protocol test"
    echo "  To run this test, install: npm install @modelcontextprotocol/sdk"
fi

kill -TERM $VANTAGE_PID 2>/dev/null
wait $VANTAGE_PID 2>/dev/null

echo "✓ MCP test completed"