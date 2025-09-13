import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Settings } from '@/types';
import apiClient from '@/api/client';

export const useSettingsStore = defineStore('settings', () => {
  // State
  const settings = ref<Settings>({
    theme: 'light',
    auto_refresh: false,
    refresh_interval: 5000,
  });
  const loading = ref(false);
  const error = ref<string | null>(null);
  const viewMode = ref<'card' | 'table'>('card');

  // Computed
  const isDarkMode = computed(() => settings.value.theme === 'dark');
  const refreshIntervalSeconds = computed(() => settings.value.refresh_interval / 1000);

  // Actions
  async function loadSettings() {
    loading.value = true;
    error.value = null;
    try {
      const serverSettings = await apiClient.getSettings();
      // Preserve local theme preference if it exists
      const localTheme = localStorage.getItem('ichimi-theme') as 'light' | 'dark' | null;
      if (localTheme) {
        serverSettings.theme = localTheme;
      }
      settings.value = serverSettings;
      applyTheme(settings.value.theme);
    } catch (e: any) {
      error.value = e.message || 'Failed to load settings';
      console.error('Failed to load settings:', e);
    } finally {
      loading.value = false;
    }
  }

  async function updateSettings(newSettings: Partial<Settings>) {
    error.value = null;
    const updatedSettings = { ...settings.value, ...newSettings };
    
    try {
      await apiClient.updateSettings(updatedSettings);
      settings.value = updatedSettings;
      
      // Apply theme if changed
      if (newSettings.theme) {
        applyTheme(newSettings.theme);
      }
    } catch (e: any) {
      error.value = e.message || 'Failed to update settings';
      throw e;
    }
  }

  function toggleTheme() {
    const newTheme = settings.value.theme === 'light' ? 'dark' : 'light';
    settings.value.theme = newTheme;
    applyTheme(newTheme);
    // Save to backend asynchronously
    updateSettings({ theme: newTheme }).catch((e) => {
      console.error('Failed to save theme preference:', e);
      // Revert on error
      const oldTheme = newTheme === 'light' ? 'dark' : 'light';
      settings.value.theme = oldTheme;
      applyTheme(oldTheme);
    });
  }

  function setViewMode(mode: 'card' | 'table') {
    viewMode.value = mode;
    // Persist to localStorage
    localStorage.setItem('ichimi-view-mode', mode);
  }

  function applyTheme(theme: 'light' | 'dark') {
    const html = document.documentElement;
    
    if (theme === 'dark') {
      html.classList.add('dark');
      html.setAttribute('data-bs-theme', 'dark');
    } else {
      html.classList.remove('dark');
      html.setAttribute('data-bs-theme', 'light');
    }
    
    // Store preference
    localStorage.setItem('ichimi-theme', theme);
  }

  function initializeSettings() {
    // Load theme from localStorage
    const savedTheme = localStorage.getItem('ichimi-theme') as 'light' | 'dark' | null;
    if (savedTheme) {
      settings.value.theme = savedTheme;
      applyTheme(savedTheme);
    } else {
      // Check system preference
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      const theme = prefersDark ? 'dark' : 'light';
      settings.value.theme = theme;
      applyTheme(theme);
    }
    
    // Load view mode from localStorage
    const savedViewMode = localStorage.getItem('ichimi-view-mode') as 'card' | 'table' | null;
    if (savedViewMode) {
      viewMode.value = savedViewMode;
    }
  }

  function clearError() {
    error.value = null;
  }

  // Watch for system theme changes
  if (window.matchMedia) {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    mediaQuery.addEventListener('change', (e) => {
      if (!localStorage.getItem('ichimi-theme')) {
        const theme = e.matches ? 'dark' : 'light';
        settings.value.theme = theme;
        applyTheme(theme);
      }
    });
  }

  return {
    // State
    settings,
    loading,
    error,
    viewMode,
    
    // Computed
    isDarkMode,
    refreshIntervalSeconds,
    
    // Actions
    loadSettings,
    updateSettings,
    toggleTheme,
    setViewMode,
    applyTheme,
    initializeSettings,
    clearError,
  };
});