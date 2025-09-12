<template>
  <div class="navbar-stats">
    <div class="container-xl">
      <div class="d-flex align-items-center">
        <!-- Total Processes -->
        <div class="stat-item">
          <span class="stat-label">Total</span>
          <span class="stat-value">{{ processStore.processCount }}</span>
        </div>
        
        <div class="stat-divider"></div>
        
        <!-- Running -->
        <div class="stat-item">
          <span class="stat-label">Running</span>
          <span class="stat-value text-green">{{ processStore.runningCount }}</span>
        </div>
        
        <div class="stat-divider"></div>
        
        <!-- Stopped -->
        <div class="stat-item">
          <span class="stat-label">Stopped</span>
          <span class="stat-value text-yellow">{{ processStore.stoppedCount }}</span>
        </div>
        
        <div class="stat-divider"></div>
        
        <!-- Failed -->
        <div class="stat-item">
          <span class="stat-label">Failed</span>
          <span class="stat-value text-red">{{ processStore.failedCount }}</span>
        </div>

        <!-- Quick Actions (Right aligned) -->
        <div class="ms-auto d-flex">
          <button 
            @click="refreshProcesses" 
            class="btn btn-sm btn-ghost-secondary"
            title="Refresh"
            :disabled="processStore.loading"
          >
            <IconRefresh :class="{ 'animate-spin': processStore.loading }" />
          </button>
          
          <button 
            @click="addTestProcesses"
            class="btn btn-sm btn-ghost-primary ms-2"
            :disabled="addingTestProcess"
            title="Add test processes for demo/testing"
          >
            <IconTestPipe /> Add Test
          </button>
          
          <button 
            @click="showCreateProcess"
            class="btn btn-sm btn-primary ms-2"
            v-if="templateStore.templateCount > 0"
          >
            <IconPlus /> New Process
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { IconRefresh, IconTestPipe, IconPlus } from '@tabler/icons-vue';
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
.animate-spin {
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
</style>