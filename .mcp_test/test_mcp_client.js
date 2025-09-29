const { spawn } = require('child_process');

// MCP request helper
function sendMcpRequest(method, params = {}) {
    return new Promise((resolve, reject) => {
        const request = {
            jsonrpc: "2.0",
            method: method,
            params: params,
            id: Date.now()
        };

        const ichimi = spawn('./target/release/ichimi', ['--no-web'], {
            stdio: ['pipe', 'pipe', 'pipe']
        });

        let responseData = '';
        let errorData = '';

        ichimi.stdout.on('data', (data) => {
            responseData += data.toString();
        });

        ichimi.stderr.on('data', (data) => {
            errorData += data.toString();
        });

        ichimi.on('close', () => {
            if (responseData) {
                try {
                    // MCPレスポンスは複数行の可能性がある
                    const lines = responseData.split('\n').filter(line => line.trim());
                    const lastLine = lines[lines.length - 1];
                    const response = JSON.parse(lastLine);
                    resolve(response);
                } catch (e) {
                    reject(new Error(`Parse error: ${e.message}\nResponse: ${responseData}`));
                }
            } else {
                reject(new Error(`No response. Error: ${errorData}`));
            }
        });

        // Send request
        ichimi.stdin.write(JSON.stringify(request) + '\n');
        ichimi.stdin.end();
    });
}

async function runTests() {
    const testDir = process.argv[2] || '.mcp_test';

    try {
        console.log('=== Ichimi MCP Self-Test ===\n');

        // Test 1: Get server status
        console.log('1. Getting server status...');
        const status = await sendMcpRequest('tools/call', {
            name: 'get_status',
            arguments: {}
        });
        console.log('Status:', JSON.stringify(status, null, 2));

        // Test 2: Create a process
        console.log('\n2. Creating test process...');
        const createResult = await sendMcpRequest('tools/call', {
            name: 'create_process',
            arguments: {
                name: 'mcp-test-process',
                command: `${testDir}/test_process.sh`,
                args: [],
                env: {},
                cwd: testDir
            }
        });
        console.log('Create result:', JSON.stringify(createResult, null, 2));

        // Test 3: List processes
        console.log('\n3. Listing processes...');
        const processes = await sendMcpRequest('tools/call', {
            name: 'list_processes',
            arguments: {}
        });
        console.log('Processes:', JSON.stringify(processes, null, 2));

    } catch (error) {
        console.error('Test failed:', error);
        process.exit(1);
    }
}

runTests();
