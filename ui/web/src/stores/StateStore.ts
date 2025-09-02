import Alpine from 'alpinejs';

// Process types
export interface Process {
  id: string;
  command: string;
  args: string[];
  cwd?: string;
  state: 'Running' | 'Stopped' | 'Failed' | 'NotStarted';
  pid?: number;
}

// Template types
export interface TemplateVariable {
  name: string;
  description?: string;
  default_value?: any;
  required?: boolean;
  var_type?: 'string' | 'number' | 'boolean' | 'path';
  validation?: {
    min?: number;
    max?: number;
  };
}

export interface ProcessTemplate {
  template_id: string;
  name: string;
  description?: string;
  category?: string;
  tags?: string[];
  variables?: TemplateVariable[];
}

export interface StateStore {
  // Settings
  mode: 'light' | 'dark';
  autoRefresh: boolean;
  refreshInterval: number;
  initialized: boolean;
  
  // Processes
  processes: Process[];
  processesLoading: boolean;
  processesError: string | null;
  autoRefreshInterval: NodeJS.Timeout | null;
  
  // Process Templates
  processTemplates: ProcessTemplate[];
  processTemplatesLoading: boolean;
  processTemplatesError: string | null;
  selectedProcessTemplate: ProcessTemplate | null;
  showProcessTemplateModal: boolean;
  processTemplateId: string;
  processTemplateVariableValues: Record<string, any>;
  
  // Settings methods
  loadSettings(): Promise<void>;
  toggleMode(): Promise<void>;
  saveSettings(): Promise<void>;
  updateTheme(): void;
  
  // Process methods
  loadProcesses(): Promise<void>;
  startProcess(id: string): Promise<void>;
  stopProcess(id: string): Promise<void>;
  removeProcess(id: string): Promise<void>;
  getStatusClass(state: string): string;
  startAutoRefresh(): void;
  stopAutoRefresh(): void;
  
  // Process Template methods
  loadProcessTemplates(): Promise<void>;
  selectProcessTemplate(template: ProcessTemplate): void;
  closeProcessTemplateModal(): void;
  instantiateProcessTemplate(): Promise<void>;
  
  // Main init
  init(): Promise<void>;
}

export const createStateStore = (): StateStore => ({
  // Settings state
  mode: 'dark',
  autoRefresh: true,
  refreshInterval: 5000,
  initialized: false,
  
  // Processes state
  processes: [],
  processesLoading: false,
  processesError: null,
  autoRefreshInterval: null,
  
  // Process Templates state
  processTemplates: [],
  processTemplatesLoading: false,
  processTemplatesError: null,
  selectedProcessTemplate: null,
  showProcessTemplateModal: false,
  processTemplateId: '',
  processTemplateVariableValues: {},

  // Settings methods
  async loadSettings() {
    try {
      const response = await fetch('/api/settings');
      if (response.ok) {
        const settings = await response.json();
        this.mode = settings.color_mode;
        this.autoRefresh = settings.auto_refresh;
        this.refreshInterval = settings.refresh_interval;
        this.updateTheme();
      }
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
      const response = await fetch('/api/settings', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          color_mode: this.mode,
          auto_refresh: this.autoRefresh,
          refresh_interval: this.refreshInterval
        })
      });
      
      if (!response.ok) {
        console.error('Failed to save settings');
      }
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
  },
  
  // Process methods
  async loadProcesses() {
    this.processesLoading = true;
    this.processesError = null;
    try {
      const response = await fetch('/api/processes');
      if (response.ok) {
        this.processes = await response.json();
      } else {
        this.processesError = 'Failed to load processes';
      }
    } catch (error) {
      this.processesError = `Error loading processes: ${error}`;
      console.error('Failed to load processes:', error);
    } finally {
      this.processesLoading = false;
    }
  },

  async startProcess(id: string) {
    try {
      const response = await fetch(`/api/processes/${id}/start`, {
        method: 'POST'
      });
      if (response.ok) {
        await this.loadProcesses();
      } else {
        const text = await response.text();
        this.processesError = `Failed to start process: ${text}`;
      }
    } catch (error) {
      this.processesError = `Error starting process: ${error}`;
      console.error('Failed to start process:', error);
    }
  },

  async stopProcess(id: string) {
    try {
      const response = await fetch(`/api/processes/${id}/stop`, {
        method: 'POST'
      });
      if (response.ok) {
        await this.loadProcesses();
      } else {
        const text = await response.text();
        this.processesError = `Failed to stop process: ${text}`;
      }
    } catch (error) {
      this.processesError = `Error stopping process: ${error}`;
      console.error('Failed to stop process:', error);
    }
  },

  async removeProcess(id: string) {
    if (!confirm(`Are you sure you want to remove process ${id}?`)) {
      return;
    }
    try {
      const response = await fetch(`/api/processes/${id}`, {
        method: 'DELETE'
      });
      if (response.ok) {
        await this.loadProcesses();
      } else {
        const text = await response.text();
        this.processesError = `Failed to remove process: ${text}`;
      }
    } catch (error) {
      this.processesError = `Error removing process: ${error}`;
      console.error('Failed to remove process:', error);
    }
  },

  getStatusClass(state: string): string {
    switch (state) {
      case 'Running':
        return 'status-running';
      case 'Stopped':
        return 'status-stopped';
      case 'Failed':
        return 'status-failed';
      case 'NotStarted':
      default:
        return 'status-notstarted';
    }
  },

  startAutoRefresh() {
    if (this.autoRefresh && !this.autoRefreshInterval) {
      this.autoRefreshInterval = setInterval(() => {
        this.loadProcesses();
      }, this.refreshInterval);
    }
  },

  stopAutoRefresh() {
    if (this.autoRefreshInterval) {
      clearInterval(this.autoRefreshInterval);
      this.autoRefreshInterval = null;
    }
  },
  
  // Process Template methods
  async loadProcessTemplates() {
    this.processTemplatesLoading = true;
    this.processTemplatesError = null;
    try {
      const response = await fetch('/api/templates');
      if (response.ok) {
        this.processTemplates = await response.json();
      } else {
        this.processTemplatesError = 'Failed to load process templates';
      }
    } catch (error) {
      this.processTemplatesError = `Error loading process templates: ${error}`;
      console.error('Failed to load process templates:', error);
    } finally {
      this.processTemplatesLoading = false;
    }
  },

  selectProcessTemplate(template: ProcessTemplate) {
    this.selectedProcessTemplate = template;
    this.showProcessTemplateModal = true;
    this.processTemplateId = `${template.template_id}-${Date.now()}`;
    this.processTemplateVariableValues = {};
    
    // Set default values
    template.variables?.forEach(variable => {
      if (variable.default_value !== undefined) {
        this.processTemplateVariableValues[variable.name] = variable.default_value;
      }
    });
  },

  closeProcessTemplateModal() {
    this.showProcessTemplateModal = false;
    this.selectedProcessTemplate = null;
    this.processTemplateId = '';
    this.processTemplateVariableValues = {};
  },

  async instantiateProcessTemplate() {
    if (!this.selectedProcessTemplate) return;
    
    try {
      const response = await fetch(`/api/templates/${this.selectedProcessTemplate.template_id}/instantiate`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          process_id: this.processTemplateId,
          values: this.processTemplateVariableValues
        })
      });

      if (response.ok) {
        // Reload processes
        await this.loadProcesses();
        this.closeProcessTemplateModal();
      } else {
        const text = await response.text();
        this.processTemplatesError = `Failed to instantiate process template: ${text}`;
      }
    } catch (error) {
      this.processTemplatesError = `Error instantiating process template: ${error}`;
      console.error('Failed to instantiate process template:', error);
    }
  },

  // Main init
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
    
    // Start auto refresh
    this.startAutoRefresh();
  }
});