import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { ProcessInfo } from '@/types';
import { isRunning, isStopped, isFailed } from '@/types';
import apiClient from '@/api/client';

export const useProcessStore = defineStore('process', () => {
  // State
  const processes = ref<ProcessInfo[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const selectedProcessId = ref<string | null>(null);

  // Computed
  const processCount = computed(() => processes.value.length);
  
  const runningCount = computed(() => 
    processes.value.filter(p => isRunning(p.state)).length
  );
  
  const stoppedCount = computed(() => 
    processes.value.filter(p => isStopped(p.state)).length
  );
  
  const failedCount = computed(() => 
    processes.value.filter(p => isFailed(p.state)).length
  );
  
  const selectedProcess = computed(() => 
    processes.value.find(p => p.id === selectedProcessId.value)
  );

  // Actions
  async function loadProcesses() {
    loading.value = true;
    error.value = null;
    try {
      processes.value = await apiClient.getProcesses();
    } catch (e: any) {
      error.value = e.message || 'Failed to load processes';
      console.error('Failed to load processes:', e);
    } finally {
      loading.value = false;
    }
  }

  async function createProcess(process: {
    id: string;
    command: string;
    args: string[];
    env?: Record<string, string>;
    cwd?: string;
  }) {
    try {
      const newProcess = await apiClient.createProcess(process);
      processes.value.push(newProcess);
      return newProcess;
    } catch (e: any) {
      error.value = e.message || 'Failed to create process';
      throw e;
    }
  }

  async function startProcess(id: string) {
    try {
      await apiClient.startProcess(id);
      await loadProcesses(); // Reload to get updated state
    } catch (e: any) {
      error.value = e.message || `Failed to start process ${id}`;
      throw e;
    }
  }

  async function stopProcess(id: string) {
    try {
      await apiClient.stopProcess(id);
      await loadProcesses(); // Reload to get updated state
    } catch (e: any) {
      error.value = e.message || `Failed to stop process ${id}`;
      throw e;
    }
  }

  async function removeProcess(id: string) {
    try {
      await apiClient.removeProcess(id);
      processes.value = processes.value.filter(p => p.id !== id);
      if (selectedProcessId.value === id) {
        selectedProcessId.value = null;
      }
    } catch (e: any) {
      error.value = e.message || `Failed to remove process ${id}`;
      throw e;
    }
  }

  async function getProcessOutput(id: string) {
    try {
      return await apiClient.getProcessOutput(id);
    } catch (e: any) {
      error.value = e.message || `Failed to get output for process ${id}`;
      throw e;
    }
  }

  async function addTestProcesses() {
    try {
      const testProcesses = await apiClient.addTestProcesses();
      processes.value.push(...testProcesses);
      return testProcesses;
    } catch (e: any) {
      error.value = e.message || 'Failed to add test processes';
      throw e;
    }
  }

  function selectProcess(id: string | null) {
    selectedProcessId.value = id;
  }

  function clearError() {
    error.value = null;
  }

  // Auto-refresh functionality
  let refreshInterval: number | null = null;

  function startAutoRefresh(interval = 5000) {
    stopAutoRefresh();
    refreshInterval = window.setInterval(() => {
      loadProcesses();
    }, interval);
  }

  function stopAutoRefresh() {
    if (refreshInterval !== null) {
      window.clearInterval(refreshInterval);
      refreshInterval = null;
    }
  }

  return {
    // State
    processes,
    loading,
    error,
    selectedProcessId,
    
    // Computed
    processCount,
    runningCount,
    stoppedCount,
    failedCount,
    selectedProcess,
    
    // Actions
    loadProcesses,
    createProcess,
    startProcess,
    stopProcess,
    removeProcess,
    getProcessOutput,
    addTestProcesses,
    selectProcess,
    clearError,
    startAutoRefresh,
    stopAutoRefresh,
  };
});