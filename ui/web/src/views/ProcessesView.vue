<template>
  <div class="processes-view">
    <div class="page-header">
      <div class="container-xl">
        <div class="header-row">
          <div class="header-title">
            <h1 class="page-title">
              Processes
              <span v-if="processStore.processCount > 0" class="stats-inline">
                · {{ stats.running }} running, {{ stats.stopped }} stopped<template v-if="stats.failed > 0">, {{ stats.failed }} failed</template>
              </span>
            </h1>
          </div>
          <div class="header-actions">
            <!-- View Mode Toggle -->
            <button
              @click="settingsStore.setViewMode('card')"
              class="action-btn"
              :class="{ active: settingsStore.viewMode === 'card' }"
              :title="t('views.card')"
              :aria-label="t('views.card')"
            >
              <IconLayoutGrid :size="16" />
            </button>
            <button
              @click="settingsStore.setViewMode('table')"
              class="action-btn"
              :class="{ active: settingsStore.viewMode === 'table' }"
              :title="t('views.table')"
              :aria-label="t('views.table')"
            >
              <IconTable :size="16" />
            </button>
            <button
              @click="processStore.loadProcesses()"
              class="action-btn"
              :disabled="processStore.loading"
              :title="t('actions.refresh')"
              :aria-label="t('actions.refresh')"
            >
              <IconRefresh :size="16" />
            </button>
          </div>
        </div>

        <!-- Filter Pills -->
        <div v-if="processStore.processCount > 0" class="filter-pills">
          <button
            @click="filterState = 'all'"
            class="filter-pill"
            :class="{ active: filterState === 'all' }"
          >
            All
          </button>
          <button
            @click="filterState = 'running'"
            class="filter-pill"
            :class="{ active: filterState === 'running' }"
          >
            Running
          </button>
          <button
            @click="filterState = 'stopped'"
            class="filter-pill"
            :class="{ active: filterState === 'stopped' }"
          >
            Stopped
          </button>
          <button
            v-if="stats.failed > 0"
            @click="filterState = 'failed'"
            class="filter-pill"
            :class="{ active: filterState === 'failed' }"
          >
            Failed
          </button>
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
import { useI18n } from 'vue-i18n';
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

const { t } = useI18n();
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

<style scoped lang="scss">
.processes-view {
  flex: 1;
}

.page-header {
  background: oklch(1 0 0);
  border-bottom: 1px solid oklch(0.92 0 0);
  padding: 1.25rem 0;

  @media (prefers-color-scheme: dark) {
    background: oklch(0.18 0 0);
    border-bottom-color: oklch(0.25 0 0);
  }
}

.header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 1rem;
}

.header-title {
  flex: 1;
}

.page-title {
  font-size: 1.5rem;
  font-weight: 700;
  margin: 0;
  color: oklch(0.2 0 0);

  @media (prefers-color-scheme: dark) {
    color: oklch(0.95 0 0);
  }

  .stats-inline {
    font-size: 0.9375rem;
    font-weight: 500;
    color: oklch(0.5 0 0);

    @media (prefers-color-scheme: dark) {
      color: oklch(0.6 0 0);
    }
  }
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: oklch(0.5 0 0);
  cursor: pointer;
  transition: all 0.15s ease;

  @media (prefers-color-scheme: dark) {
    color: oklch(0.65 0 0);
  }

  &:hover:not(:disabled) {
    background: oklch(0.96 0 0);
    color: oklch(0.2 0 0);

    @media (prefers-color-scheme: dark) {
      background: oklch(0.22 0 0);
      color: oklch(0.9 0 0);
    }
  }

  &.active {
    background: var(--vantage-btn-primary-bg);
    color: oklch(1 0 0);
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
}

.filter-pills {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.filter-pill {
  padding: 0.375rem 0.875rem;
  border: none;
  border-radius: 999px;
  background: oklch(0.96 0 0);
  color: oklch(0.4 0 0);
  font-size: 0.8125rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;

  @media (prefers-color-scheme: dark) {
    background: oklch(0.22 0 0);
    color: oklch(0.7 0 0);
  }

  &:hover:not(.active) {
    background: oklch(0.94 0 0);

    @media (prefers-color-scheme: dark) {
      background: oklch(0.24 0 0);
    }
  }

  &.active {
    background: var(--vantage-btn-primary-bg);
    color: oklch(1 0 0);
  }
}

.page-body {
  padding: 1.5rem 0;
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

.empty {
  padding: 3rem 1rem;
  text-align: center;
}

.empty-title {
  font-size: 1.125rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: oklch(0.3 0 0);

  @media (prefers-color-scheme: dark) {
    color: oklch(0.8 0 0);
  }
}

.empty-subtitle {
  font-size: 0.875rem;
  margin-bottom: 1.5rem;
  color: oklch(0.5 0 0);

  @media (prefers-color-scheme: dark) {
    color: oklch(0.6 0 0);
  }
}

.spinner-border {
  margin: 3rem auto;
  display: block;
}

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

@media (max-width: 768px) {
  .page-header {
    padding: 1rem 0;
  }

  .header-row {
    margin-bottom: 0.75rem;
  }

  .page-title {
    font-size: 1.25rem;

    .stats-inline {
      display: block;
      font-size: 0.8125rem;
      margin-top: 0.25rem;
    }
  }

  .page-body {
    padding: 1rem 0;
  }
}
</style>