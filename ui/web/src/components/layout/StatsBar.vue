<template>
  <div class="vantage-stats">
    <div class="container-xl">
      <div class="stats-content">
        <!-- Statistics Cards -->
        <div class="stats-grid">
          <!-- Total -->
          <div class="stat-card stat-card-total">
            <div class="stat-icon">
              <IconChartPie :size="20" :stroke-width="2" />
            </div>
            <div class="stat-content">
              <div class="stat-label">Total</div>
              <div class="stat-value">{{ processStore.processCount }}</div>
            </div>
          </div>

          <!-- Running -->
          <div class="stat-card stat-card-running">
            <div class="stat-icon">
              <IconPlayerPlay :size="20" :stroke-width="2" />
            </div>
            <div class="stat-content">
              <div class="stat-label">Running</div>
              <div class="stat-value">{{ processStore.runningCount }}</div>
            </div>
          </div>

          <!-- Stopped -->
          <div class="stat-card stat-card-stopped">
            <div class="stat-icon">
              <IconPlayerPause :size="20" :stroke-width="2" />
            </div>
            <div class="stat-content">
              <div class="stat-label">Stopped</div>
              <div class="stat-value">{{ processStore.stoppedCount }}</div>
            </div>
          </div>

          <!-- Failed -->
          <div class="stat-card stat-card-failed" v-if="processStore.failedCount > 0">
            <div class="stat-icon">
              <IconAlertCircle :size="20" :stroke-width="2" />
            </div>
            <div class="stat-content">
              <div class="stat-label">Failed</div>
              <div class="stat-value">{{ processStore.failedCount }}</div>
            </div>
          </div>
        </div>

        <!-- Quick Actions -->
        <div class="stats-actions">
          <button
            @click="refreshProcesses"
            class="action-btn action-btn-icon"
            :class="{ 'action-btn-loading': processStore.loading }"
            title="Refresh"
            :disabled="processStore.loading"
          >
            <IconRefresh :size="18" :stroke-width="2" />
          </button>

          <button
            @click="addTestProcesses"
            class="action-btn action-btn-secondary"
            :disabled="addingTestProcess"
            title="Add test processes for demo/testing"
          >
            <IconTestPipe :size="18" :stroke-width="2" />
            <span class="action-label">Add Test</span>
          </button>

          <button
            @click="showCreateProcess"
            class="action-btn action-btn-primary"
            v-if="templateStore.templateCount > 0"
          >
            <IconPlus :size="18" :stroke-width="2" />
            <span class="action-label">New Process</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import {
  IconRefresh,
  IconTestPipe,
  IconPlus,
  IconChartPie,
  IconPlayerPlay,
  IconPlayerPause,
  IconAlertCircle,
} from '@tabler/icons-vue';
import { useProcessStore } from '@/stores/process';
import { useTemplateStore } from '@/stores/template';

const router = useRouter();
const processStore = useProcessStore();
const templateStore = useTemplateStore();

const addingTestProcess = ref(false);

async function refreshProcesses() {
  await processStore.loadProcesses();
}

async function addTestProcesses() {
  addingTestProcess.value = true;
  try {
    await processStore.addTestProcesses();
  } catch (error) {
    console.error('Failed to add test processes:', error);
  } finally {
    addingTestProcess.value = false;
  }
}

function showCreateProcess() {
  router.push({ name: 'templates' });
}
</script>

<style scoped lang="scss">
.vantage-stats {
  background: oklch(0.97 0 0);
  border-bottom: 1px solid oklch(0.92 0 0);

  @media (prefers-color-scheme: dark) {
    background: oklch(0.16 0 0);
    border-bottom-color: oklch(0.22 0 0);
  }
}

.stats-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 2rem;
  padding: 1rem 0;
}

// Stats Grid
.stats-grid {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-wrap: wrap;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  background: oklch(1 0 0);
  border-radius: 8px;
  border: 1px solid oklch(0.92 0 0);
  transition: all 0.15s ease;
  min-width: 120px;

  @media (prefers-color-scheme: dark) {
    background: oklch(0.2 0 0);
    border-color: oklch(0.28 0 0);
  }

  &:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px oklch(0 0 0 / 0.08);

    @media (prefers-color-scheme: dark) {
      box-shadow: 0 4px 12px oklch(0 0 0 / 0.3);
    }
  }
}

.stat-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: 8px;
  transition: all 0.15s ease;
}

// Total Card
.stat-card-total .stat-icon {
  background: oklch(0.85 0.1 240 / 0.2);
  color: oklch(0.5 0.15 240);

  @media (prefers-color-scheme: dark) {
    background: oklch(0.3 0.1 240 / 0.3);
    color: oklch(0.7 0.15 240);
  }
}

// Running Card
.stat-card-running .stat-icon {
  background: var(--vantage-btn-green-bg);
  color: oklch(1 0 0);
}

.stat-card-running .stat-value {
  color: var(--vantage-btn-green-bg);
}

// Stopped Card
.stat-card-stopped .stat-icon {
  background: var(--vantage-btn-yellow-bg);
  color: oklch(1 0 0);
}

.stat-card-stopped .stat-value {
  color: var(--vantage-btn-yellow-bg);
}

// Failed Card
.stat-card-failed .stat-icon {
  background: var(--vantage-btn-red-bg);
  color: oklch(1 0 0);
}

.stat-card-failed .stat-value {
  color: var(--vantage-btn-red-bg);
}

.stat-content {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.stat-label {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: oklch(0.55 0 0);

  @media (prefers-color-scheme: dark) {
    color: oklch(0.6 0 0);
  }
}

.stat-value {
  font-size: 1.5rem;
  font-weight: 700;
  line-height: 1;
  color: oklch(0.25 0 0);

  @media (prefers-color-scheme: dark) {
    color: oklch(0.9 0 0);
  }
}

// Actions
.stats-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-shrink: 0;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 6px;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  &:not(:disabled):hover {
    transform: translateY(-1px);
  }

  &:not(:disabled):active {
    transform: translateY(0);
  }
}

.action-btn-icon {
  padding: 0.5rem;
  background: transparent;
  color: oklch(0.5 0 0);

  @media (prefers-color-scheme: dark) {
    color: oklch(0.65 0 0);
  }

  &:not(:disabled):hover {
    background: oklch(0.95 0 0);
    color: oklch(0.3 0 0);

    @media (prefers-color-scheme: dark) {
      background: oklch(0.25 0 0);
      color: oklch(0.85 0 0);
    }
  }
}

.action-btn-loading {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.action-btn-secondary {
  background: oklch(0.96 0 0);
  color: oklch(0.4 0 0);
  border: 1px solid oklch(0.9 0 0);

  @media (prefers-color-scheme: dark) {
    background: oklch(0.22 0 0);
    color: oklch(0.75 0 0);
    border-color: oklch(0.3 0 0);
  }

  &:not(:disabled):hover {
    background: oklch(0.94 0 0);
    border-color: oklch(0.85 0 0);

    @media (prefers-color-scheme: dark) {
      background: oklch(0.26 0 0);
      border-color: oklch(0.35 0 0);
    }
  }
}

.action-btn-primary {
  background: var(--vantage-btn-primary-bg);
  color: oklch(1 0 0);

  &:not(:disabled):hover {
    background: var(--vantage-btn-primary-hover);
  }

  &:not(:disabled):active {
    background: var(--vantage-btn-primary-active);
  }
}

// Responsive Design
@media (max-width: 992px) {
  .stats-content {
    gap: 1.5rem;
  }

  .stat-card {
    padding: 0.625rem 0.875rem;
    min-width: 100px;
  }

  .stat-icon {
    width: 36px;
    height: 36px;
  }

  .stat-value {
    font-size: 1.375rem;
  }
}

@media (max-width: 768px) {
  .stats-content {
    flex-direction: column;
    gap: 1rem;
    align-items: stretch;
  }

  .stats-grid {
    justify-content: space-between;
  }

  .stat-card {
    flex: 1;
    min-width: 0;
  }

  .stats-actions {
    width: 100%;
    justify-content: flex-end;
  }

  .action-label {
    display: none;
  }

  .action-btn:not(.action-btn-icon) {
    padding: 0.5rem;
  }
}

@media (max-width: 576px) {
  .stat-label {
    font-size: 0.625rem;
  }

  .stat-value {
    font-size: 1.25rem;
  }

  .stat-icon {
    width: 32px;
    height: 32px;
  }

  .stat-card {
    padding: 0.5rem 0.625rem;
    gap: 0.5rem;
  }
}
</style>