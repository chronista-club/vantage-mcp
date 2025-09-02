import { createSettingsStore, SettingsStore } from './SettingsStore';
import { createProcessStore, ProcessStore } from './ProcessStore';
import { createProcessInfoStore, ProcessInfoStore } from './ProcessInfoStore';
import { createTestProcessStore, TestProcessStore } from './TestProcessStore';

export interface MainStore extends SettingsStore, ProcessStore, ProcessInfoStore, TestProcessStore {
  initialized: boolean;
  init(): Promise<void>;
}

export const createMainStore = (): MainStore => {
  // Create individual stores
  const settingsStore = createSettingsStore();
  const processStore = createProcessStore();
  const processInfoStore = createProcessInfoStore(async () => {
    await processStore.loadProcesses();
  });
  const testProcessStore = createTestProcessStore(async () => {
    await processStore.loadProcesses();
  });

  // Create combined store
  const mainStore: MainStore = {
    // Spread all store properties
    ...settingsStore,
    ...processStore,
    ...processInfoStore,
    ...testProcessStore,
    
    // Main state
    initialized: false,

    // Main init method
    async init() {
      if (this.initialized) {
        console.warn('State store init() called multiple times - skipping');
        return;
      }

      this.initialized = true;
      console.log('State init');

      // Load all initial data
      await Promise.all([
        this.loadSettings(),
        this.loadProcesses(),
        this.loadProcessTemplates()
      ]);

      // Start auto-refresh if enabled
      if (this.autoRefresh) {
        this.startAutoRefresh(this.refreshInterval);
      }
    }
  };

  // Bind methods to the mainStore context
  Object.setPrototypeOf(mainStore, {
    ...Object.getPrototypeOf(settingsStore),
    ...Object.getPrototypeOf(processStore),
    ...Object.getPrototypeOf(processInfoStore),
    ...Object.getPrototypeOf(testProcessStore),
  });

  return mainStore;
};