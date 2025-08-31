<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { processes, processCount } from '$lib/stores/processes';
  import ProcessCard from '$lib/components/ProcessCard.svelte';
  import CreateProcessModal from '$lib/components/CreateProcessModal.svelte';
  import LogViewer from '$lib/components/LogViewer.svelte';
  import type { ProcessConfig } from '$lib/types';
  
  let showCreateModal = false;
  let showLogsModal = false;
  let selectedProcessId = '';
  let theme: 'light' | 'dark' = 'light';
  
  // Load theme from localStorage
  onMount(() => {
    const savedTheme = localStorage.getItem('theme') as 'light' | 'dark' | null;
    if (savedTheme) {
      theme = savedTheme;
      document.documentElement.setAttribute('data-bs-theme', theme);
    }
    
    processes.load();
    processes.startAutoRefresh(5000);
  });
  
  onDestroy(() => {
    processes.stopAutoRefresh();
  });
  
  function toggleTheme() {
    theme = theme === 'light' ? 'dark' : 'light';
    localStorage.setItem('theme', theme);
    document.documentElement.setAttribute('data-bs-theme', theme);
  }
  
  async function handleCreate(event: CustomEvent<ProcessConfig>) {
    try {
      await processes.create(event.detail);
      showCreateModal = false;
    } catch (error) {
      alert(`Failed to create process: ${error}`);
    }
  }
  
  async function handleProcessAction(action: string, id: string) {
    try {
      switch(action) {
        case 'start':
          await processes.start(id);
          break;
        case 'stop':
          await processes.stop(id);
          break;
        case 'restart':
          await processes.restart(id);
          break;
        case 'remove':
          await processes.remove(id);
          break;
      }
    } catch (error) {
      alert(`Failed to ${action} process: ${error}`);
    }
  }
  
  function showLogs(id: string) {
    selectedProcessId = id;
    showLogsModal = true;
  }
</script>

<div class="page">
  <!-- Navbar -->
  <header class="navbar navbar-expand-md d-print-none">
    <div class="container-xl">
      <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbar-menu">
        <span class="navbar-toggler-icon"></span>
      </button>
      <h1 class="navbar-brand navbar-brand-autodark d-none-navbar-horizontal pe-0 pe-md-3">
        <a href="/">
          <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon text-primary">
            <circle cx="12" cy="5" r="3"/>
            <circle cx="19" cy="12" r="3"/>
            <circle cx="5" cy="12" r="3"/>
            <circle cx="12" cy="19" r="3"/>
            <line x1="12" y1="8" x2="12" y2="16"/>
            <line x1="9.5" y1="6.5" x2="7" y2="10"/>
            <line x1="14.5" y1="6.5" x2="17" y2="10"/>
            <line x1="7" y1="14" x2="9.5" y2="17.5"/>
            <line x1="17" y1="14" x2="14.5" y2="17.5"/>
          </svg>
          Ichimi Server
        </a>
      </h1>
      <div class="navbar-nav flex-row order-md-last">
        <div class="d-none d-md-flex">
          <button on:click={toggleTheme} class="nav-link px-0" title="Toggle theme">
            {#if theme === 'light'}
              <i class="ti ti-moon fs-2"></i>
            {:else}
              <i class="ti ti-sun fs-2"></i>
            {/if}
          </button>
        </div>
      </div>
    </div>
  </header>
  
  <!-- Page wrapper -->
  <div class="page-wrapper">
    <!-- Page header -->
    <div class="page-header d-print-none">
      <div class="container-xl">
        <div class="row g-2 align-items-center">
          <div class="col">
            <div class="page-pretitle">Dashboard</div>
            <h2 class="page-title">Process Manager</h2>
          </div>
          <div class="col-auto ms-auto d-print-none">
            <div class="btn-list">
              <button 
                class="btn btn-primary d-none d-sm-inline-block"
                on:click={() => showCreateModal = true}
              >
                <i class="ti ti-plus"></i>
                Create process
              </button>
              <button 
                class="btn btn-primary d-sm-none btn-icon"
                on:click={() => showCreateModal = true}
              >
                <i class="ti ti-plus"></i>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Page body -->
    <div class="page-body">
      <div class="container-xl">
        <!-- Stats -->
        <div class="row row-deck row-cards mb-4">
          <div class="col-sm-6 col-lg-3">
            <div class="card">
              <div class="card-body">
                <div class="d-flex align-items-center">
                  <div class="subheader">Total Processes</div>
                </div>
                <div class="h1 mb-0">{$processCount.total}</div>
              </div>
              <div class="card-footer bg-success-lt">
                <i class="ti ti-list-check me-1"></i> All processes
              </div>
            </div>
          </div>
          
          <div class="col-sm-6 col-lg-3">
            <div class="card">
              <div class="card-body">
                <div class="d-flex align-items-center">
                  <div class="subheader">Running</div>
                </div>
                <div class="h1 mb-0 text-success">{$processCount.running}</div>
              </div>
              <div class="card-footer bg-success-lt">
                <i class="ti ti-player-play-filled me-1"></i> Active now
              </div>
            </div>
          </div>
          
          <div class="col-sm-6 col-lg-3">
            <div class="card">
              <div class="card-body">
                <div class="d-flex align-items-center">
                  <div class="subheader">Stopped</div>
                </div>
                <div class="h1 mb-0 text-secondary">{$processCount.stopped}</div>
              </div>
              <div class="card-footer bg-secondary-lt">
                <i class="ti ti-player-stop-filled me-1"></i> Inactive
              </div>
            </div>
          </div>
          
          <div class="col-sm-6 col-lg-3">
            <div class="card">
              <div class="card-body">
                <div class="d-flex align-items-center">
                  <div class="subheader">Failed</div>
                </div>
                <div class="h1 mb-0 text-danger">{$processCount.failed}</div>
              </div>
              <div class="card-footer bg-danger-lt">
                <i class="ti ti-alert-circle me-1"></i> Need attention
              </div>
            </div>
          </div>
        </div>
        
        <!-- Process list -->
        {#if $processes.length === 0}
          <div class="empty">
            <div class="empty-img">
              <img src="data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIxMjgiIGhlaWdodD0iMTI4IiB2aWV3Qm94PSIwIDAgMjQgMjQiPjxwYXRoIGZpbGw9Im5vbmUiIHN0cm9rZT0iY3VycmVudENvbG9yIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMiIgZD0iTTEyIDEyYTggOCAwIDEgMCAwLTE2YTggOCAwIDAgMCAwIDE2eiIvPjxwYXRoIGZpbGw9Im5vbmUiIHN0cm9rZT0iY3VycmVudENvbG9yIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMiIgZD0iTTggMTJoOG0tNCAwdjgiLz48L3N2Zz4=" alt="Empty" height="128">
            </div>
            <p class="empty-title">No processes yet</p>
            <p class="empty-subtitle text-muted">
              Create your first process to get started with Ichimi Server
            </p>
            <div class="empty-action">
              <button class="btn btn-primary" on:click={() => showCreateModal = true}>
                <i class="ti ti-plus"></i>
                Create your first process
              </button>
            </div>
          </div>
        {:else}
          <div class="row row-cards">
            {#each $processes as process (process.id)}
              <div class="col-sm-6 col-lg-4">
                <ProcessCard 
                  {process}
                  on:start={(e) => handleProcessAction('start', e.detail.id)}
                  on:stop={(e) => handleProcessAction('stop', e.detail.id)}
                  on:restart={(e) => handleProcessAction('restart', e.detail.id)}
                  on:remove={(e) => handleProcessAction('remove', e.detail.id)}
                  on:viewLogs={(e) => showLogs(e.detail.id)}
                />
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
    
    <!-- Footer -->
    <footer class="footer footer-transparent d-print-none">
      <div class="container-xl">
        <div class="row text-center align-items-center flex-row-reverse">
          <div class="col-lg-auto ms-lg-auto">
            <ul class="list-inline list-inline-dots mb-0">
              <li class="list-inline-item">
                <a href="https://github.com/chronista-inc/ichimi" class="link-secondary" rel="noopener">
                  <i class="ti ti-brand-github"></i> Source code
                </a>
              </li>
            </ul>
          </div>
          <div class="col-12 col-lg-auto mt-3 mt-lg-0">
            <ul class="list-inline list-inline-dots mb-0">
              <li class="list-inline-item">
                &copy; 2024 Ichimi Server
              </li>
              <li class="list-inline-item">
                Version 0.1.0-beta15
              </li>
            </ul>
          </div>
        </div>
      </div>
    </footer>
  </div>
</div>

<!-- Modals -->
{#if showCreateModal}
  <CreateProcessModal 
    on:close={() => showCreateModal = false}
    on:create={handleCreate}
  />
{/if}

{#if showLogsModal}
  <LogViewer 
    processId={selectedProcessId}
    onClose={() => showLogsModal = false}
  />
{/if}