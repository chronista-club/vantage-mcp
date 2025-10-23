<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Process } from '$lib/types';

  export let process: Process;
  
  const dispatch = createEventDispatcher();
  
  function handleStart() {
    dispatch('start', { id: process.id });
  }
  
  function handleStop() {
    dispatch('stop', { id: process.id });
  }
  
  function handleRestart() {
    dispatch('restart', { id: process.id });
  }
  
  function handleRemove() {
    if (confirm(`Are you sure you want to remove "${process.id}"?`)) {
      dispatch('remove', { id: process.id });
    }
  }
  
  function viewLogs() {
    dispatch('viewLogs', { id: process.id });
  }
  
  $: statusColor = {
    'running': 'success',
    'stopped': 'secondary',
    'failed': 'danger',
    'not_started': 'warning'
  }[process.state] || 'secondary';
  
  $: statusIcon = {
    'running': 'player-play-filled',
    'stopped': 'player-stop-filled',
    'failed': 'alert-circle',
    'not_started': 'clock'
  }[process.state] || 'help';
</script>

<div class="card">
  <div class="card-header">
    <h3 class="card-title">{process.id}</h3>
    <div class="card-actions">
      <span class="badge bg-{statusColor}">
        <i class="ti ti-{statusIcon} me-1"></i>
        {process.state.replace('_', ' ')}
      </span>
    </div>
  </div>
  
  <div class="card-body">
    <div class="mb-2">
      <span class="text-muted">Command:</span>
      <code class="ms-2">{process.command} {process.args.join(' ')}</code>
    </div>
    
    {#if process.cwd}
      <div class="mb-2">
        <span class="text-muted">Directory:</span>
        <code class="ms-2">{process.cwd}</code>
      </div>
    {/if}
    
    {#if process.pid}
      <div class="mb-2">
        <span class="text-muted">PID:</span>
        <strong class="ms-2">{process.pid}</strong>
      </div>
    {/if}
    
    {#if process.auto_start_on_restore}
      <div class="mb-2">
        <i class="ti ti-refresh text-info"></i>
        <span class="text-muted ms-1">Auto-start on restore</span>
      </div>
    {/if}
    
    {#if process.error_message}
      <div class="alert alert-danger mb-0 mt-2">
        <i class="ti ti-alert-circle me-1"></i>
        {process.error_message}
      </div>
    {/if}
  </div>
  
  <div class="card-footer">
    <div class="btn-list">
      {#if process.state === 'running'}
        <button class="btn btn-warning btn-sm" on:click={handleRestart}>
          <i class="ti ti-refresh"></i> Restart
        </button>
        <button class="btn btn-danger btn-sm" on:click={handleStop}>
          <i class="ti ti-player-stop"></i> Stop
        </button>
      {:else}
        <button class="btn btn-success btn-sm" on:click={handleStart}>
          <i class="ti ti-player-play"></i> Start
        </button>
        <button class="btn btn-outline-danger btn-sm" on:click={handleRemove}>
          <i class="ti ti-trash"></i> Remove
        </button>
      {/if}
      
      <button class="btn btn-info btn-sm" on:click={viewLogs}>
        <i class="ti ti-file-text"></i> Logs
      </button>
    </div>
  </div>
</div>