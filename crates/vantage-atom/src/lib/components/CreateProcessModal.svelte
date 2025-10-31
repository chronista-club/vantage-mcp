<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import type { ProcessConfig } from '$lib/types';

  const dispatch = createEventDispatcher();
  
  let modal: HTMLDivElement;
  let form: HTMLFormElement;
  
  let config: ProcessConfig = {
    id: '',
    command: '',
    args: [],
    env: {},
    cwd: '',
    auto_start_on_restore: false
  };
  
  let argsText = '';
  let envText = '';
  
  onMount(() => {
    // Show modal
    modal.classList.add('show');
    modal.style.display = 'block';
    document.body.classList.add('modal-open');
    
    // Focus first input
    const firstInput = form.querySelector('input');
    if (firstInput) firstInput.focus();
  });
  
  function close() {
    modal.classList.remove('show');
    modal.style.display = 'none';
    document.body.classList.remove('modal-open');
    setTimeout(() => dispatch('close'), 300);
  }
  
  function handleSubmit() {
    // Parse args
    config.args = argsText
      .split(/\s+/)
      .filter(arg => arg.length > 0);
    
    // Parse env
    config.env = {};
    envText.split('\n').forEach(line => {
      const [key, value] = line.split('=');
      if (key && value) {
        config.env[key.trim()] = value.trim();
      }
    });
    
    // Clean up empty cwd
    if (!config.cwd) {
      delete config.cwd;
    }
    
    dispatch('create', config);
    close();
  }
  
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      close();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="modal modal-blur fade" bind:this={modal} tabindex="-1">
  <div class="modal-dialog modal-lg">
    <div class="modal-content">
      <div class="modal-header">
        <h5 class="modal-title">Create New Process</h5>
        <button type="button" class="btn-close" on:click={close}></button>
      </div>
      
      <form bind:this={form} on:submit|preventDefault={handleSubmit}>
        <div class="modal-body">
          <div class="mb-3">
            <label class="form-label required">Process ID</label>
            <input 
              type="text" 
              class="form-control" 
              bind:value={config.id}
              placeholder="my-process"
              required
            />
            <small class="form-hint">Unique identifier for the process</small>
          </div>
          
          <div class="mb-3">
            <label class="form-label required">Command</label>
            <input 
              type="text" 
              class="form-control" 
              bind:value={config.command}
              placeholder="npm"
              required
            />
            <small class="form-hint">The executable to run</small>
          </div>
          
          <div class="mb-3">
            <label class="form-label">Arguments</label>
            <input 
              type="text" 
              class="form-control" 
              bind:value={argsText}
              placeholder="run dev --port 3000"
            />
            <small class="form-hint">Space-separated arguments</small>
          </div>
          
          <div class="mb-3">
            <label class="form-label">Working Directory</label>
            <input 
              type="text" 
              class="form-control" 
              bind:value={config.cwd}
              placeholder="/path/to/project"
            />
            <small class="form-hint">Leave empty to use current directory</small>
          </div>
          
          <div class="mb-3">
            <label class="form-label">Environment Variables</label>
            <textarea 
              class="form-control" 
              rows="3"
              bind:value={envText}
              placeholder="NODE_ENV=production&#10;PORT=3000"
            ></textarea>
            <small class="form-hint">One per line in KEY=VALUE format</small>
          </div>
          
          <div class="mb-3">
            <label class="form-check">
              <input 
                type="checkbox" 
                class="form-check-input"
                bind:checked={config.auto_start_on_restore}
              />
              <span class="form-check-label">
                Auto-start on server restore
              </span>
            </label>
          </div>
        </div>
        
        <div class="modal-footer">
          <button type="button" class="btn me-auto" on:click={close}>
            Cancel
          </button>
          <button type="submit" class="btn btn-primary">
            <i class="ti ti-plus me-1"></i>
            Create Process
          </button>
        </div>
      </form>
    </div>
  </div>
</div>

<div class="modal-backdrop fade show"></div>