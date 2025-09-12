<template>
  <header class="navbar navbar-expand-md navbar-light d-print-none">
    <div class="container-xl">
      <h1 class="navbar-brand navbar-brand-autodark d-none-navbar-horizontal pe-0 pe-md-3">
        <router-link to="/" class="text-decoration-none">
          🚀 Ichimi Server
        </router-link>
      </h1>
      
      <div class="navbar-nav flex-row order-md-last">
        <!-- Theme Toggle -->
        <div class="nav-item">
          <button 
            @click="toggleTheme"
            class="btn btn-ghost-secondary btn-icon"
            :title="`Switch to ${isDarkMode ? 'light' : 'dark'} mode`"
          >
            <IconSun v-if="isDarkMode" />
            <IconMoon v-else />
          </button>
        </div>
        
        <!-- Settings Button -->
        <div class="nav-item ms-2">
          <button 
            @click="showSettings"
            class="btn btn-ghost-secondary btn-icon"
            title="Settings"
          >
            <IconSettings />
          </button>
        </div>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { IconSun, IconMoon, IconSettings } from '@tabler/icons-vue';
import { useSettingsStore } from '@/stores/settings';
import { computed } from 'vue';

const settingsStore = useSettingsStore();

const isDarkMode = computed(() => settingsStore.isDarkMode);

function toggleTheme() {
  settingsStore.toggleTheme();
}

function showSettings() {
  // TODO: Implement settings modal
  console.log('Show settings');
}
</script>