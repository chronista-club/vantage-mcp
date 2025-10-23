<template>
  <div class="stats-bar bg-white border-bottom">
    <div class="container-xl">
      <div class="d-flex align-items-center py-3">
        <!-- Statistics Cards -->
        <div class="d-flex gap-3 flex-wrap">
          <!-- Total -->
          <div class="stat-card">
            <div class="stat-icon bg-azure-lt text-azure">
              <IconChartPie />
            </div>
            <div class="stat-content">
              <div class="stat-label">Total</div>
              <div class="stat-value">{{ processStore.processCount }}</div>
            </div>
          </div>

          <!-- Running -->
          <div class="stat-card">
            <div class="stat-icon bg-green-lt text-green">
              <IconPlayerPlay />
            </div>
            <div class="stat-content">
              <div class="stat-label">Running</div>
              <div class="stat-value text-green">{{ processStore.runningCount }}</div>
            </div>
          </div>

          <!-- Stopped -->
          <div class="stat-card">
            <div class="stat-icon bg-secondary-lt text-secondary">
              <IconPlayerPause />
            </div>
            <div class="stat-content">
              <div class="stat-label">Stopped</div>
              <div class="stat-value text-secondary">{{ processStore.stoppedCount }}</div>
            </div>
          </div>

          <!-- Failed -->
          <div class="stat-card" v-if="processStore.failedCount > 0">
            <div class="stat-icon bg-red-lt text-red">
              <IconAlertCircle />
            </div>
            <div class="stat-content">
              <div class="stat-label">Failed</div>
              <div class="stat-value text-red">{{ processStore.failedCount }}</div>
            </div>
          </div>
        </div>

        <!-- Quick Actions (Right aligned) -->
        <div class="ms-auto d-flex gap-2">
          <button
            @click="refreshProcesses"
            class="btn btn-icon"
            :class="processStore.loading ? 'btn-loading' : ''"
            title="Refresh"
            :disabled="processStore.loading"
          >
            <IconRefresh />
          </button>

          <button
            @click="addTestProcesses"
            class="btn btn-sm btn-outline-primary"
            :disabled="addingTestProcess"
            title="Add test processes for demo/testing"
          >
            <IconTestPipe class="icon" />
            <span class="d-none d-md-inline ms-2">Add Test</span>
          </button>

          <button
            @click="showCreateProcess"
            class="btn btn-sm btn-primary"
            v-if="templateStore.templateCount > 0"
          >
            <IconPlus class="icon" />
            <span class="ms-2">New Process</span>
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

<style scoped>
.stats-bar {
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem 1rem;
  background: white;
  border-radius: 0.5rem;
  border: 1px solid var(--tblr-border-color);
  transition: all 0.2s ease;
}

.stat-card:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.stat-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.5rem;
  height: 2.5rem;
  border-radius: 0.375rem;
  font-size: 1.25rem;
}

.stat-content {
  display: flex;
  flex-direction: column;
}

.stat-label {
  font-size: 0.75rem;
  color: var(--tblr-secondary);
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.stat-value {
  font-size: 1.5rem;
  font-weight: 700;
  line-height: 1;
}

.btn-icon .icon,
.btn .icon {
  width: 1.125rem;
  height: 1.125rem;
}

@media (max-width: 992px) {
  .stat-card {
    padding: 0.375rem 0.75rem;
  }

  .stat-icon {
    width: 2rem;
    height: 2rem;
    font-size: 1rem;
  }

  .stat-value {
    font-size: 1.25rem;
  }
}

@media (max-width: 576px) {
  .stat-label {
    font-size: 0.65rem;
  }

  .stat-value {
    font-size: 1.125rem;
  }
}
</style>