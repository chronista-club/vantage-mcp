/// <reference types="alpinejs" />

console.log('app.ts loaded');

// このファイルをモジュールとして扱うためのexport
export {};

// カラーモードのenum定義
enum ColorMode {
  Light = 'light',
  Dark = 'dark'
}

// プロセス状態のenum定義
enum ProcessState {
  NotStarted = 'NotStarted',
  Running = 'Running',
  Stopped = 'Stopped',
  Failed = 'Failed',
}

// プロセス情報の型定義
interface ProcessInfo {
  id: string;
  command: string;
  args: string[];
  env: Record<string, string>;
  cwd?: string;
  state: ProcessState;
  auto_start_on_restore: boolean;
  pid?: number;
}

// ストアの型定義
interface StateStore {
  mode: ColorMode;
  autoRefresh: boolean;
  refreshInterval: number;
  initialized: boolean;
  loadSettings(): Promise<void>;
  toggleMode(): Promise<void>;
  saveSettings(): Promise<void>;
  updateTheme(): void;
  init(): Promise<void>;
}

interface ProcessesStore {
  processes: ProcessInfo[];
  loading: boolean;
  error: string | null;
  refreshInterval: number | null;
  initialized: boolean;
  loadProcesses(): Promise<void>;
  startProcess(id: string): Promise<void>;
  stopProcess(id: string): Promise<void>;
  removeProcess(id: string): Promise<void>;
  getStatusClass(state: ProcessState): string;
  startAutoRefresh(): void;
  stopAutoRefresh(): void;
  init(): Promise<void>;
}

// Alpine型を拡張
declare global {
  interface Window {
    Alpine: {
      store<T>(name: string, store?: T): T | void;
      data(name: string, component: () => any): void;
      plugin(plugin: any): void;
    };
  }
}

declare module 'alpinejs' {
  interface Stores {
    state: StateStore;
    processes: ProcessesStore;
  }
}

// Alpine.js初期化時の処理
document.addEventListener('alpine:init', () => {
  console.log('Alpine.js is initializing...');
  
  // グローバルステートストア
  const stateStore: StateStore = {
    mode: ColorMode.Dark,
    autoRefresh: true,
    refreshInterval: 5000,
    initialized: false,
    
    async loadSettings() {
      try {
        const response = await fetch('/api/settings');
        if (response.ok) {
          const settings = await response.json();
          this.mode = settings.color_mode as ColorMode;
          this.autoRefresh = settings.auto_refresh;
          this.refreshInterval = settings.refresh_interval;
          this.updateTheme();
        }
      } catch (error) {
        console.error('Failed to load settings:', error);
      }
    },
    
    async toggleMode() {
      this.mode = this.mode === ColorMode.Dark ? ColorMode.Light : ColorMode.Dark;
      // DOMにクラスを適用
      this.updateTheme();
      // サーバーに保存
      await this.saveSettings();
    },
    
    async saveSettings() {
      try {
        const response = await fetch('/api/settings', {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            color_mode: this.mode,
            auto_refresh: this.autoRefresh,
            refresh_interval: this.refreshInterval,
          }),
        });
        
        if (!response.ok) {
          console.error('Failed to save settings');
        }
      } catch (error) {
        console.error('Failed to save settings:', error);
      }
    },
    
    updateTheme() {
      // ダークモードの場合はクラスを追加、ライトモードの場合は削除
      if (this.mode === ColorMode.Dark) {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
    },
    
    async init() {
      if (this.initialized) {
        console.warn('State store init() called multiple times - skipping');
        return;
      }
      this.initialized = true;
      console.log('State init');
      // サーバーから設定を読み込み
      await this.loadSettings();
    }
  };
  
  window.Alpine.store('state', stateStore);
  
  // Stateストアの初期化（一度だけ実行）
  stateStore.init();
  
  // プロセス管理ストア
  const processesStore: ProcessesStore = {
    processes: [] as ProcessInfo[],
    loading: false,
    error: null as string | null,
    refreshInterval: null as number | null,
    initialized: false,
    
    async loadProcesses() {
      this.loading = true;
      this.error = null;
      
      try {
        const response = await fetch('/api/processes');
        if (response.ok) {
          this.processes = await response.json();
        } else {
          this.error = 'Failed to load processes';
        }
      } catch (error) {
        this.error = `Error loading processes: ${error}`;
        console.error('Failed to load processes:', error);
      } finally {
        this.loading = false;
      }
    },
    
    async startProcess(id: string) {
      try {
        const response = await fetch(`/api/processes/${id}/start`, {
          method: 'POST',
        });
        
        if (response.ok) {
          await this.loadProcesses();
        } else {
          const error = await response.text();
          this.error = `Failed to start process: ${error}`;
        }
      } catch (error) {
        this.error = `Error starting process: ${error}`;
        console.error('Failed to start process:', error);
      }
    },
    
    async stopProcess(id: string) {
      try {
        const response = await fetch(`/api/processes/${id}/stop`, {
          method: 'POST',
        });
        
        if (response.ok) {
          await this.loadProcesses();
        } else {
          const error = await response.text();
          this.error = `Failed to stop process: ${error}`;
        }
      } catch (error) {
        this.error = `Error stopping process: ${error}`;
        console.error('Failed to stop process:', error);
      }
    },
    
    async removeProcess(id: string) {
      if (!confirm(`Are you sure you want to remove process ${id}?`)) {
        return;
      }
      
      try {
        const response = await fetch(`/api/processes/${id}`, {
          method: 'DELETE',
        });
        
        if (response.ok) {
          await this.loadProcesses();
        } else {
          const error = await response.text();
          this.error = `Failed to remove process: ${error}`;
        }
      } catch (error) {
        this.error = `Error removing process: ${error}`;
        console.error('Failed to remove process:', error);
      }
    },
    
    getStatusClass(state: ProcessState): string {
      switch (state) {
        case ProcessState.Running:
          return 'status-running';
        case ProcessState.Stopped:
          return 'status-stopped';
        case ProcessState.Failed:
          return 'status-failed';
        case ProcessState.NotStarted:
        default:
          return 'status-notstarted';
      }
    },
    
    startAutoRefresh() {
      const stateStore = window.Alpine.store('state') as any;
      if (stateStore.autoRefresh && !this.refreshInterval) {
        this.refreshInterval = setInterval(() => {
          this.loadProcesses();
        }, stateStore.refreshInterval);
      }
    },
    
    stopAutoRefresh() {
      if (this.refreshInterval) {
        clearInterval(this.refreshInterval);
        this.refreshInterval = null;
      }
    },
    
    async init() {
      if (this.initialized) {
        console.warn('Processes store init() called multiple times - skipping');
        return;
      }
      this.initialized = true;
      console.log('Processes store init');
      await this.loadProcesses();
      this.startAutoRefresh();
    },
  };
  
  window.Alpine.store('processes', processesStore);
  
  // Processesストアの初期化（一度だけ実行）
  processesStore.init();
});

console.log('Event listener registered');