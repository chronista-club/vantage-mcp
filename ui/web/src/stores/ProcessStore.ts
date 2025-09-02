import { Process } from '../types';
import { apiClient } from '../api/client';

export interface ProcessStore {
  processes: Process[];
  processesLoading: boolean;
  processesError: string | null;
  autoRefreshInterval: NodeJS.Timeout | null;
  
  loadProcesses(): Promise<void>;
  startProcess(id: string): Promise<void>;
  stopProcess(id: string): Promise<void>;
  removeProcess(id: string): Promise<void>;
  getStatusClass(state: string): string;
  startAutoRefresh(interval: number): void;
  stopAutoRefresh(): void;
}

export const createProcessStore = (): ProcessStore => ({
  processes: [],
  processesLoading: false,
  processesError: null,
  autoRefreshInterval: null,

  async loadProcesses() {
    this.processesLoading = true;
    this.processesError = null;

    try {
      this.processes = await apiClient.getProcesses();
    } catch (error) {
      this.processesError = error instanceof Error ? error.message : 'Failed to load processes';
      console.error('Failed to load processes:', error);
    } finally {
      this.processesLoading = false;
    }
  },

  async startProcess(id: string) {
    try {
      await apiClient.startProcess(id);
      await this.loadProcesses();
    } catch (error) {
      this.processesError = error instanceof Error ? error.message : 'Failed to start process';
      console.error('Failed to start process:', error);
    }
  },

  async stopProcess(id: string) {
    try {
      await apiClient.stopProcess(id);
      await this.loadProcesses();
    } catch (error) {
      this.processesError = error instanceof Error ? error.message : 'Failed to stop process';
      console.error('Failed to stop process:', error);
    }
  },

  async removeProcess(id: string) {
    if (!confirm(`Are you sure you want to remove process ${id}?`)) {
      return;
    }

    try {
      await apiClient.removeProcess(id);
      await this.loadProcesses();
    } catch (error) {
      this.processesError = error instanceof Error ? error.message : 'Failed to remove process';
      console.error('Failed to remove process:', error);
    }
  },

  getStatusClass(state: any): string {
    if (state === 'NotStarted') {
      return 'status-notstarted';
    } else if (state?.Running) {
      return 'status-running';
    } else if (state?.Stopped) {
      return 'status-stopped';
    } else if (state?.Failed) {
      return 'status-failed';
    }
    return 'status-notstarted';
  },

  startAutoRefresh(interval: number) {
    if (this.autoRefreshInterval) {
      return;
    }
    
    this.autoRefreshInterval = setInterval(() => {
      this.loadProcesses();
    }, interval);
  },

  stopAutoRefresh() {
    if (this.autoRefreshInterval) {
      clearInterval(this.autoRefreshInterval);
      this.autoRefreshInterval = null;
    }
  }
});