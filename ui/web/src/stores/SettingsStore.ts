import { Settings } from '../types';
import { apiClient } from '../api/client';

export interface SettingsStore {
  mode: 'light' | 'dark';
  autoRefresh: boolean;
  refreshInterval: number;
  
  loadSettings(): Promise<void>;
  toggleMode(): Promise<void>;
  saveSettings(): Promise<void>;
  updateTheme(): void;
}

export const createSettingsStore = (): SettingsStore => ({
  mode: 'dark',
  autoRefresh: true,
  refreshInterval: 5000,

  async loadSettings() {
    try {
      const settings = await apiClient.getSettings();
      this.mode = settings.color_mode;
      this.autoRefresh = settings.auto_refresh;
      this.refreshInterval = settings.refresh_interval;
      this.updateTheme();
    } catch (error) {
      console.error('Failed to load settings:', error);
    }
  },

  async toggleMode() {
    this.mode = this.mode === 'dark' ? 'light' : 'dark';
    this.updateTheme();
    await this.saveSettings();
  },

  async saveSettings() {
    try {
      await apiClient.updateSettings({
        color_mode: this.mode,
        auto_refresh: this.autoRefresh,
        refresh_interval: this.refreshInterval
      });
    } catch (error) {
      console.error('Failed to save settings:', error);
    }
  },

  updateTheme() {
    if (this.mode === 'dark') {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }
});