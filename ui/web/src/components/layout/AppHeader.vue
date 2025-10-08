<template>
  <header class="navbar navbar-expand-md d-print-none sticky-top bg-white border-bottom">
    <div class="container-xl">
      <h1 class="navbar-brand navbar-brand-autodark d-none-navbar-horizontal pe-0 pe-md-3 mb-0">
        <router-link to="/" class="text-decoration-none d-flex align-items-center">
          <span class="brand-logo me-2">🚀</span>
          <span class="brand-text">
            <span class="fw-bold">Ichimi</span>
            <span class="text-muted ms-1">Server</span>
          </span>
        </router-link>
      </h1>

      <div class="navbar-nav flex-row order-md-last">
        <!-- Theme Toggle -->
        <div class="nav-item">
          <button
            @click="toggleTheme"
            class="btn btn-icon"
            :class="isDarkMode ? 'btn-icon-light' : 'btn-icon-dark'"
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
            class="btn btn-icon btn-icon-secondary"
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

<style scoped>
.brand-logo {
  font-size: 1.5rem;
  line-height: 1;
  transition: transform 0.2s ease;
}

.navbar-brand:hover .brand-logo {
  transform: scale(1.1);
}

.brand-text {
  font-size: 1.25rem;
  line-height: 1;
}

.btn-icon {
  transition: all 0.2s ease;
}

.btn-icon:hover {
  transform: translateY(-1px);
}

.navbar {
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}
</style>