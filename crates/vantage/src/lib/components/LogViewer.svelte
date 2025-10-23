<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { api } from '$lib/api/client';
  import type { ProcessOutput } from '$lib/types';

  export let processId: string;
  export let onClose: () => void;
  
  let logs: string[] = [];
  let loading = true;
  let autoScroll = true;
  let logContainer: HTMLElement;
  let refreshInterval: number;
  
  onMount(async () => {
    await loadLogs();
    
    // Auto-refresh logs every 2 seconds
    refreshInterval = window.setInterval(loadLogs, 2000);
  });
  
  onDestroy(() => {
    if (refreshInterval) {
      clearInterval(refreshInterval);
    }
  });
  
  async function loadLogs() {
    try {
      const output = await api.getProcessOutput(processId);
      logs = output.lines;
      loading = false;
      
      if (autoScroll && logContainer) {
        setTimeout(() => {
          logContainer.scrollTop = logContainer.scrollHeight;
        }, 0);
      }
    } catch (error) {
      console.error('Failed to load logs:', error);
      loading = false;
    }
  }
  
  function handleClear() {
    logs = [];
  }
  
  function handleCopy() {
    const text = logs.join('\n');
    navigator.clipboard.writeText(text);
  }
</script>

<div class="modal modal-blur fade show" style="display: block;">
  <div class="modal-dialog modal-xl">
    <div class="modal-content">
      <div class="modal-header">
        <h5 class="modal-title">Process Logs: {processId}</h5>
        <button type="button" class="btn-close" on:click={onClose}></button>
      </div>
      
      <div class="modal-body p-0">
        <div class="d-flex align-items-center justify-content-between px-3 py-2 border-bottom">
          <div class="form-check form-switch">
            <input 
              type="checkbox" 
              class="form-check-input" 
              id="autoScroll"
              bind:checked={autoScroll}
            />
            <label class="form-check-label" for="autoScroll">
              Auto-scroll
            </label>
          </div>
          
          <div class="btn-group">
            <button class="btn btn-sm btn-outline-secondary" on:click={handleCopy}>
              <i class="ti ti-copy"></i> Copy
            </button>
            <button class="btn btn-sm btn-outline-secondary" on:click={handleClear}>
              <i class="ti ti-trash"></i> Clear
            </button>
            <button class="btn btn-sm btn-outline-primary" on:click={loadLogs}>
              <i class="ti ti-refresh"></i> Refresh
            </button>
          </div>
        </div>
        
        <div 
          class="log-viewer" 
          bind:this={logContainer}
          style="height: 500px; overflow-y: auto; background: #1a1a1a; color: #e4e4e4; font-family: 'Cascadia Code', 'Fira Code', monospace; font-size: 13px; line-height: 1.5;"
        >
          {#if loading}
            <div class="p-3 text-center">
              <div class="spinner-border spinner-border-sm text-light" role="status"></div>
              <span class="ms-2">Loading logs...</span>
            </div>
          {:else if logs.length === 0}
            <div class="p-3 text-center text-muted">
              No logs available
            </div>
          {:else}
            <pre class="m-0 p-3">{logs.join('\n')}</pre>
          {/if}
        </div>
      </div>
      
      <div class="modal-footer">
        <button type="button" class="btn" on:click={onClose}>
          Close
        </button>
      </div>
    </div>
  </div>
</div>

<div class="modal-backdrop fade show"></div>

<style>
  .log-viewer pre {
    white-space: pre-wrap;
    word-wrap: break-word;
  }
  
  .log-viewer::-webkit-scrollbar {
    width: 8px;
  }
  
  .log-viewer::-webkit-scrollbar-track {
    background: #2a2a2a;
  }
  
  .log-viewer::-webkit-scrollbar-thumb {
    background: #555;
    border-radius: 4px;
  }
  
  .log-viewer::-webkit-scrollbar-thumb:hover {
    background: #777;
  }
</style>