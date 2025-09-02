import { apiClient } from '../api/client';

export interface TestProcessStore {
  addingTestProcess: boolean;
  
  addTestProcesses(): Promise<void>;
}

export const createTestProcessStore = (onProcessCreated: () => Promise<void>): TestProcessStore => ({
  addingTestProcess: false,

  async addTestProcesses() {
    this.addingTestProcess = true;
    
    const testProcesses = [
      {
        id: `echo-test-${Date.now()}`,
        command: 'echo',
        args: ['Hello from Ichimi Server!'],
        env: {},
        cwd: null
      },
      {
        id: `sleep-test-${Date.now()}`,
        command: 'sleep',
        args: ['300'],
        env: {},
        cwd: null
      },
      {
        id: `date-loop-${Date.now()}`,
        command: 'sh',
        args: ['-c', 'while true; do date; echo "---"; sleep 5; done'],
        env: {},
        cwd: null
      },
      {
        id: `python-server-${Date.now()}`,
        command: 'python3',
        args: ['-m', 'http.server', '0'],  // Use port 0 for random available port
        env: {},
        cwd: '/tmp'
      },
      {
        id: `counter-${Date.now()}`,
        command: 'sh',
        args: ['-c', 'for i in $(seq 1 100); do echo "Count: $i"; sleep 1; done'],
        env: {},
        cwd: null
      }
    ];

    try {
      // Create all test processes
      for (const process of testProcesses) {
        await apiClient.createProcess(process);
      }
      
      // Reload processes after creation
      await onProcessCreated();
      
      console.log('Test processes added successfully');
    } catch (error) {
      console.error('Failed to add test processes:', error);
    } finally {
      this.addingTestProcess = false;
    }
  }
});