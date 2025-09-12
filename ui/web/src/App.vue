<template>
  <div class="page">
    <AppHeader />
    <NavigationBar />
    <StatsBar />
    
    <div class="page-wrapper">
      <router-view v-slot="{ Component }">
        <transition name="fade" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import AppHeader from '@/components/layout/AppHeader.vue';
import NavigationBar from '@/components/layout/NavigationBar.vue';
import StatsBar from '@/components/layout/StatsBar.vue';
import { useSettingsStore } from '@/stores/settings';
import { useProcessStore } from '@/stores/process';
import { useTemplateStore } from '@/stores/template';

const settingsStore = useSettingsStore();
const processStore = useProcessStore();
const templateStore = useTemplateStore();

onMounted(async () => {
  // Initialize settings (theme, etc.)
  settingsStore.initializeSettings();
  
  // Load initial data
  await Promise.all([
    processStore.loadProcesses(),
    templateStore.loadTemplates(),
    settingsStore.loadSettings().catch(() => {
      // Settings may not be available yet, that's ok
    }),
  ]);
});
</script>

<style lang="scss">
@use '@/styles/main.scss' as *;
</style>