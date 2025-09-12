import { writable, derived } from 'svelte/store';
import { api } from '$lib/api/client';
import type { Process, ProcessConfig } from '$lib/types';

function createProcessStore() {
  const { subscribe, set, update } = writable<Process[]>([]);
  let refreshInterval: number | null = null;

  return {
    subscribe,

    async load() {
      try {
        const processes = await api.listProcesses();
        set(processes);
      } catch (error) {
        console.error('Failed to load processes:', error);
      }
    },

    async create(config: ProcessConfig) {
      await api.createProcess(config);
      await this.load();
    },

    async start(id: string) {
      await api.startProcess(id);
      await this.load();
    },

    async stop(id: string) {
      await api.stopProcess(id);
      await this.load();
    },

    async restart(id: string) {
      await api.restartProcess(id);
      await this.load();
    },

    async remove(id: string) {
      await api.removeProcess(id);
      update(list => list.filter(p => p.id !== id));
    },

    startAutoRefresh(intervalMs: number = 5000) {
      this.stopAutoRefresh();
      refreshInterval = window.setInterval(() => {
        this.load();
      }, intervalMs);
    },

    stopAutoRefresh() {
      if (refreshInterval) {
        clearInterval(refreshInterval);
        refreshInterval = null;
      }
    }
  };
}

export const processes = createProcessStore();

// Derived stores
export const runningProcesses = derived(
  processes,
  $processes => $processes.filter(p => p.state === 'running')
);

export const processCount = derived(
  processes,
  $processes => ({
    total: $processes.length,
    running: $processes.filter(p => p.state === 'running').length,
    stopped: $processes.filter(p => p.state === 'stopped').length,
    failed: $processes.filter(p => p.state === 'failed').length,
    not_started: $processes.filter(p => p.state === 'not_started').length,
  })
);