<template>
  <div>
    <div class="page-header d-print-none">
      <div class="container-xl">
        <div class="row g-2 align-items-center">
          <div class="col">
            <h2 class="page-title">Dashboard</h2>
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
          v-else-if="!processStore.loading && processStore.processCount === 0" 
          class="empty"
        >
          <p class="empty-title">No processes found</p>
          <p class="empty-subtitle text-muted">Create your first process from a template</p>
          <div class="empty-action">
            <router-link :to="{ name: 'templates' }" class="btn btn-primary">
              <IconPlus /> Create Process
            </router-link>
          </div>
        </div>

        <!-- Process Cards -->
        <div 
          v-else
          class="row row-cards"
        >
          <div 
            v-for="process in processStore.processes" 
            :key="process.id"
            class="col-12"
          >
            <ProcessCard :process="process" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { IconPlus } from '@tabler/icons-vue';
import ProcessCard from '@/components/process/ProcessCard.vue';
import { useProcessStore } from '@/stores/process';
import { useSettingsStore } from '@/stores/settings';

const processStore = useProcessStore();
const settingsStore = useSettingsStore();

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