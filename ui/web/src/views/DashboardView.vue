<template>
  <div class="dashboard-view">
    <div class="container-xl">
      <div class="page-header">
        <h1 class="page-title">{{ t('navigation.dashboard') }}</h1>
      </div>

      <!-- Statistics Cards -->
      <div class="stats-grid">
        <div class="stat-card">
          <div class="stat-icon total">
            <IconActivity :size="24" :stroke-width="2" />
          </div>
          <div class="stat-content">
            <div class="stat-label">{{ t('stats.total') }}</div>
            <div class="stat-value">{{ stats.total }}</div>
          </div>
        </div>

        <div class="stat-card">
          <div class="stat-icon running">
            <IconPlayerPlay :size="24" :stroke-width="2" />
          </div>
          <div class="stat-content">
            <div class="stat-label">{{ t('stats.running') }}</div>
            <div class="stat-value">{{ stats.running }}</div>
          </div>
        </div>

        <div class="stat-card">
          <div class="stat-icon stopped">
            <IconPlayerPause :size="24" :stroke-width="2" />
          </div>
          <div class="stat-content">
            <div class="stat-label">{{ t('stats.stopped') }}</div>
            <div class="stat-value">{{ stats.stopped }}</div>
          </div>
        </div>

        <div class="stat-card">
          <div class="stat-icon failed">
            <IconAlertTriangle :size="24" :stroke-width="2" />
          </div>
          <div class="stat-content">
            <div class="stat-label">{{ t('stats.failed') }}</div>
            <div class="stat-value">{{ stats.failed }}</div>
          </div>
        </div>
      </div>

      <!-- Quick Actions -->
      <div class="quick-actions">
        <h2 class="section-title">{{ t('dashboard.quickActions') }}</h2>
        <div class="action-grid">
          <router-link to="/processes" class="action-card">
            <IconActivity :size="20" :stroke-width="2" />
            <span>{{ t('navigation.processes') }}</span>
          </router-link>
          <router-link to="/templates" class="action-card">
            <IconTemplate :size="20" :stroke-width="2" />
            <span>{{ t('navigation.templates') }}</span>
          </router-link>
          <router-link to="/clipboard" class="action-card">
            <IconClipboard :size="20" :stroke-width="2" />
            <span>{{ t('navigation.clipboard') }}</span>
          </router-link>
        </div>
      </div>

      <!-- Recent Processes -->
      <div v-if="recentProcesses.length > 0" class="recent-section">
        <div class="section-header">
          <h2 class="section-title">{{ t('dashboard.recentProcesses') }}</h2>
          <router-link to="/processes" class="view-all-link">
            {{ t('dashboard.viewAll') }}
            <IconChevronRight :size="16" :stroke-width="2" />
          </router-link>
        </div>
        <div class="process-list">
          <div
            v-for="process in recentProcesses"
            :key="process.id"
            class="process-item"
          >
            <div class="process-info">
              <div class="process-name">{{ process.id }}</div>
              <div class="process-command">{{ process.command }}</div>
            </div>
            <div class="process-status" :class="getProcessStateClass(process.state)">
              {{ t(`status.${getProcessStateClass(process.state)}`) }}
            </div>
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else class="empty-state">
        <IconActivity :size="48" :stroke-width="1.5" />
        <h3>{{ t('dashboard.noProcesses') }}</h3>
        <p>{{ t('dashboard.noProcessesDescription') }}</p>
        <router-link to="/templates" class="btn-primary">
          {{ t('dashboard.createFromTemplate') }}
        </router-link>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useProcessStore } from '@/stores/process';
import { isRunning, isStopped, isFailed, getStateLabel } from '@/types';
import {
  IconActivity,
  IconPlayerPlay,
  IconPlayerPause,
  IconAlertTriangle,
  IconTemplate,
  IconClipboard,
  IconChevronRight,
} from '@tabler/icons-vue';

const { t } = useI18n();
const processStore = useProcessStore();

const stats = computed(() => {
  const processes = processStore.processes;
  return {
    total: processes.length,
    running: processes.filter((p) => isRunning(p.state)).length,
    stopped: processes.filter((p) => isStopped(p.state)).length,
    failed: processes.filter((p) => isFailed(p.state)).length,
  };
});

const recentProcesses = computed(() => {
  return processStore.processes.slice(0, 5);
});

function getProcessStateClass(state: any): string {
  const label = getStateLabel(state);
  return label.toLowerCase().replace(' ', '');
}
</script>

<style scoped lang="scss">
.dashboard-view {
  padding: 2rem 0;
}

.page-header {
  margin-bottom: 2rem;
}

.page-title {
  font-size: 1.75rem;
  font-weight: 700;
  color: oklch(0.2 0 0);
  margin: 0;

  @media (prefers-color-scheme: dark) {
    color: oklch(0.95 0 0);
  }
}

// Statistics grid
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 1rem;
  margin-bottom: 2rem;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1.25rem;
  background: oklch(1 0 0);
  border: 1px solid oklch(0.92 0 0);
  border-radius: 8px;
  transition: all 0.15s ease;

  @media (prefers-color-scheme: dark) {
    background: oklch(0.2 0 0);
    border-color: oklch(0.3 0 0);
  }

  &:hover {
    border-color: oklch(0.85 0 0);
    box-shadow: 0 2px 8px oklch(0 0 0 / 0.05);

    @media (prefers-color-scheme: dark) {
      border-color: oklch(0.35 0 0);
      box-shadow: 0 2px 8px oklch(0 0 0 / 0.2);
    }
  }
}

.stat-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  border-radius: 8px;
  flex-shrink: 0;

  &.total {
    background: oklch(0.95 0.02 260 / 0.4);
    color: var(--vantage-btn-primary-bg);

    @media (prefers-color-scheme: dark) {
      background: oklch(0.25 0.04 260 / 0.3);
    }
  }

  &.running {
    background: oklch(0.95 0.02 150 / 0.4);
    color: oklch(0.5 0.15 150);

    @media (prefers-color-scheme: dark) {
      background: oklch(0.25 0.04 150 / 0.3);
      color: oklch(0.7 0.15 150);
    }
  }

  &.stopped {
    background: oklch(0.95 0.02 60 / 0.4);
    color: oklch(0.55 0.12 60);

    @media (prefers-color-scheme: dark) {
      background: oklch(0.25 0.04 60 / 0.3);
      color: oklch(0.75 0.12 60);
    }
  }

  &.failed {
    background: oklch(0.95 0.02 30 / 0.4);
    color: oklch(0.55 0.18 30);

    @media (prefers-color-scheme: dark) {
      background: oklch(0.25 0.04 30 / 0.3);
      color: oklch(0.75 0.18 30);
    }
  }
}

.stat-content {
  flex: 1;
  min-width: 0;
}

.stat-label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: oklch(0.5 0 0);
  margin-bottom: 0.25rem;

  @media (prefers-color-scheme: dark) {
    color: oklch(0.65 0 0);
  }
}

.stat-value {
  font-size: 1.75rem;
  font-weight: 700;
  color: oklch(0.2 0 0);
  line-height: 1;

  @media (prefers-color-scheme: dark) {
    color: oklch(0.95 0 0);
  }
}

// Sections
.quick-actions,
.recent-section {
  margin-bottom: 2rem;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1rem;
}

.section-title {
  font-size: 1.25rem;
  font-weight: 600;
  color: oklch(0.2 0 0);
  margin: 0 0 1rem;

  @media (prefers-color-scheme: dark) {
    color: oklch(0.95 0 0);
  }
}

.view-all-link {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--vantage-btn-primary-bg);
  text-decoration: none;
  transition: opacity 0.15s ease;

  &:hover {
    opacity: 0.8;
  }
}

// Quick actions
.action-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 1rem;
}

.action-card {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 1rem;
  background: oklch(1 0 0);
  border: 1px solid oklch(0.92 0 0);
  border-radius: 8px;
  color: oklch(0.3 0 0);
  font-weight: 500;
  text-decoration: none;
  transition: all 0.15s ease;

  @media (prefers-color-scheme: dark) {
    background: oklch(0.2 0 0);
    border-color: oklch(0.3 0 0);
    color: oklch(0.85 0 0);
  }

  &:hover {
    border-color: var(--vantage-btn-primary-bg);
    color: var(--vantage-btn-primary-bg);
    transform: translateY(-2px);
    box-shadow: 0 4px 12px oklch(0 0 0 / 0.08);

    @media (prefers-color-scheme: dark) {
      box-shadow: 0 4px 12px oklch(0 0 0 / 0.3);
    }
  }
}

// Process list
.process-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.process-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem;
  background: oklch(1 0 0);
  border: 1px solid oklch(0.92 0 0);
  border-radius: 8px;
  transition: all 0.15s ease;

  @media (prefers-color-scheme: dark) {
    background: oklch(0.2 0 0);
    border-color: oklch(0.3 0 0);
  }

  &:hover {
    border-color: oklch(0.85 0 0);

    @media (prefers-color-scheme: dark) {
      border-color: oklch(0.35 0 0);
    }
  }
}

.process-info {
  flex: 1;
  min-width: 0;
}

.process-name {
  font-weight: 600;
  color: oklch(0.2 0 0);
  margin-bottom: 0.25rem;

  @media (prefers-color-scheme: dark) {
    color: oklch(0.95 0 0);
  }
}

.process-command {
  font-size: 0.8125rem;
  color: oklch(0.5 0 0);
  font-family: var(--vantage-font-monospace);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;

  @media (prefers-color-scheme: dark) {
    color: oklch(0.65 0 0);
  }
}

.process-status {
  padding: 0.375rem 0.75rem;
  border-radius: 6px;
  font-size: 0.8125rem;
  font-weight: 500;
  flex-shrink: 0;

  &.running {
    background: oklch(0.95 0.02 150 / 0.4);
    color: oklch(0.4 0.15 150);

    @media (prefers-color-scheme: dark) {
      background: oklch(0.25 0.04 150 / 0.3);
      color: oklch(0.75 0.15 150);
    }
  }

  &.stopped {
    background: oklch(0.95 0.02 60 / 0.4);
    color: oklch(0.45 0.12 60);

    @media (prefers-color-scheme: dark) {
      background: oklch(0.25 0.04 60 / 0.3);
      color: oklch(0.75 0.12 60);
    }
  }

  &.failed {
    background: oklch(0.95 0.02 30 / 0.4);
    color: oklch(0.45 0.18 30);

    @media (prefers-color-scheme: dark) {
      background: oklch(0.25 0.04 30 / 0.3);
      color: oklch(0.75 0.18 30);
    }
  }
}

// Empty state
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4rem 2rem;
  text-align: center;

  svg {
    color: oklch(0.7 0 0);
    margin-bottom: 1.5rem;

    @media (prefers-color-scheme: dark) {
      color: oklch(0.5 0 0);
    }
  }

  h3 {
    font-size: 1.25rem;
    font-weight: 600;
    color: oklch(0.3 0 0);
    margin: 0 0 0.5rem;

    @media (prefers-color-scheme: dark) {
      color: oklch(0.85 0 0);
    }
  }

  p {
    font-size: 0.9375rem;
    color: oklch(0.5 0 0);
    margin: 0 0 1.5rem;

    @media (prefers-color-scheme: dark) {
      color: oklch(0.65 0 0);
    }
  }

  .btn-primary {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1.25rem;
    background: var(--vantage-btn-primary-bg);
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 0.875rem;
    font-weight: 500;
    text-decoration: none;
    cursor: pointer;
    transition: all 0.15s ease;

    &:hover {
      background: var(--vantage-btn-primary-bg-hover);
      transform: translateY(-1px);
      box-shadow: 0 4px 12px oklch(0.6 0.2 260 / 0.3);
    }

    &:active {
      transform: translateY(0);
    }
  }
}

@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .action-grid {
    grid-template-columns: 1fr;
  }
}
</style>
