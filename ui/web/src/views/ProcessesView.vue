<template>
  <div>
    <div class="page-header d-print-none">
      <div class="container-xl">
        <div class="row g-2 align-items-center">
          <div class="col">
            <h2 class="page-title">Processes</h2>
            <!-- Statistics Summary -->
            <div class="text-muted mt-1">
              <span class="me-3">
                <span class="badge bg-success-lt">{{ stats.running }}</span> Running
              </span>
              <span class="me-3">
                <span class="badge bg-secondary-lt">{{ stats.stopped }}</span> Stopped
              </span>
              <span v-if="stats.failed > 0">
                <span class="badge bg-danger-lt">{{ stats.failed }}</span> Failed
              </span>
            </div>
          </div>
          <div class="col-auto ms-auto d-print-none">
            <div class="d-flex gap-2">
              <!-- State Filter -->
              <div class="btn-group" role="group">
                <button
                  @click="filterState = 'all'"
                  class="btn btn-sm"
                  :class="filterState === 'all' ? 'btn-primary' : 'btn-outline-primary'"
                >
                  All
                </button>
                <button
                  @click="filterState = 'running'"
                  class="btn btn-sm"
                  :class="filterState === 'running' ? 'btn-success' : 'btn-outline-success'"
                >
                  Running
                </button>
                <button
                  @click="filterState = 'stopped'"
                  class="btn btn-sm"
                  :class="filterState === 'stopped' ? 'btn-secondary' : 'btn-outline-secondary'"
                >
                  Stopped
                </button>
                <button
                  @click="filterState = 'failed'"
                  class="btn btn-sm"
                  :class="filterState === 'failed' ? 'btn-danger' : 'btn-outline-danger'"
                >
                  Failed
                </button>
              </div>
              <!-- View Mode Toggle -->
              <div class="btn-group" role="group">
                <button
                  @click="settingsStore.setViewMode('card')"
                  class="btn btn-sm btn-icon"
                  :class="settingsStore.viewMode === 'card' ? 'btn-primary' : 'btn-outline-primary'"
                  title="Card View"
                >
                  <IconLayoutGrid />
                </button>
                <button
                  @click="settingsStore.setViewMode('table')"
                  class="btn btn-sm btn-icon"
                  :class="settingsStore.viewMode === 'table' ? 'btn-primary' : 'btn-outline-primary'"
                  title="Table View"
                >
                  <IconTable />
                </button>
              </div>
              <button @click="processStore.loadProcesses()" class="btn btn-sm btn-primary">
                <IconRefresh class="icon" />
                <span class="d-none d-lg-inline ms-1">Refresh</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="page-body">
      <div class="container-xl">
        <!-- Error Alert -->
        <div v-if="processStore.error" class="alert alert-danger">
          {{ processStore.error }}
        </div>

        <!-- Loading State -->
        <div v-if="processStore.loading" class="text-center">
          <div class="spinner-border" role="status"></div>
        </div>

        <!-- Empty State -->
        <div
          v-else-if="!processStore.loading && filteredProcesses.length === 0"
          class="empty"
        >
          <p class="empty-title">No processes found</p>
          <p class="empty-subtitle text-muted">
            {{ processStore.processCount === 0 ? 'Create your first process from a template' : 'No processes match the current filter' }}
          </p>
          <div class="empty-action">
            <router-link v-if="processStore.processCount === 0" :to="{ name: 'templates' }" class="btn btn-primary">
              <IconPlus /> Create Process
            </router-link>
            <button v-else @click="filterState = 'all'" class="btn btn-primary">
              Show All Processes
            </button>
          </div>
        </div>

        <!-- Card View -->
        <TransitionGroup
          v-if="settingsStore.viewMode === 'card'"
          name="process-list"
          tag="div"
          class="row row-cards"
        >
          <div
            v-for="process in filteredProcesses"
            :key="process.id"
            class="col-12 col-md-6 col-xl-4"
          >
            <ProcessCard :process="process" />
          </div>
        </TransitionGroup>

        <!-- Table View -->
        <div
          v-else-if="settingsStore.viewMode === 'table'"
          class="card"
        >
          <ProcessTable :processes="filteredProcesses" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import {
  IconPlus,
  IconRefresh,
  IconLayoutGrid,
  IconTable
} from '@tabler/icons-vue';
import ProcessCard from '@/components/process/ProcessCard.vue';
import ProcessTable from '@/components/process/ProcessTable.vue';
import { useProcessStore } from '@/stores/process';
import { useSettingsStore } from '@/stores/settings';
import { isRunning, isStopped, isFailed, isNotStarted } from '@/types';

const processStore = useProcessStore();
const settingsStore = useSettingsStore();

// Filter state
const filterState = ref<'all' | 'running' | 'stopped' | 'failed'>('all');

// Filtered processes based on state
const filteredProcesses = computed(() => {
  if (filterState.value === 'all') {
    return processStore.processes;
  }

  return processStore.processes.filter((process) => {
    switch (filterState.value) {
      case 'running':
        return isRunning(process.state);
      case 'stopped':
        return isStopped(process.state) || isNotStarted(process.state);
      case 'failed':
        return isFailed(process.state);
      default:
        return true;
    }
  });
});

// Statistics
const stats = computed(() => {
  const running = processStore.processes.filter((p) => isRunning(p.state)).length;
  const stopped = processStore.processes.filter(
    (p) => isStopped(p.state) || isNotStarted(p.state)
  ).length;
  const failed = processStore.processes.filter((p) => isFailed(p.state)).length;

  return { running, stopped, failed };
});

onMounted(async () => {
  await processStore.loadProcesses();

  // Start auto-refresh if enabled
  if (settingsStore.settings.auto_refresh) {
    processStore.startAutoRefresh(settingsStore.settings.refresh_interval);
  }
});

onUnmounted(() => {
  processStore.stopAutoRefresh();
});
</script>

<style scoped>
.page-header {
  background: white;
  border-bottom: 1px solid var(--tblr-border-color);
  padding: 1.5rem 0;
  margin-bottom: 1.5rem;
}

.page-title {
  font-weight: 600;
  margin-bottom: 0.5rem;
}

.btn-group .btn {
  transition: all 0.2s ease;
}

.btn-group .btn:hover {
  transform: translateY(-1px);
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon {
  width: 1.125rem;
  height: 1.125rem;
}

.row-cards {
  margin-left: -0.5rem;
  margin-right: -0.5rem;
}

.row-cards > [class*='col-'] {
  padding-left: 0.5rem;
  padding-right: 0.5rem;
  margin-bottom: 1rem;
}

/* Empty state */
.empty {
  padding: 3rem 1rem;
  text-align: center;
}

.empty-title {
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
}

.empty-subtitle {
  font-size: 0.875rem;
  margin-bottom: 1.5rem;
}

/* Loading state */
.spinner-border {
  margin: 3rem auto;
  display: block;
}

/* Process list transitions */
.process-list-move,
.process-list-enter-active,
.process-list-leave-active {
  transition: all 0.3s ease;
}

.process-list-enter-from {
  opacity: 0;
  transform: translateY(20px);
}

.process-list-leave-to {
  opacity: 0;
  transform: translateY(-20px);
}

.process-list-leave-active {
  position: absolute;
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .page-header {
    padding: 1rem 0;
    margin-bottom: 1rem;
  }

  .btn-group {
    flex-wrap: nowrap;
  }
}
</style>