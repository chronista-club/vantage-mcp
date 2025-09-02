import { Process, ProcessTemplate, Settings } from '../types';

class ApiClient {
  private baseUrl = '/api';

  // Settings API
  async getSettings(): Promise<Settings> {
    const response = await fetch(`${this.baseUrl}/settings`);
    if (!response.ok) throw new Error('Failed to load settings');
    return response.json();
  }

  async updateSettings(settings: Settings): Promise<void> {
    const response = await fetch(`${this.baseUrl}/settings`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(settings)
    });
    if (!response.ok) throw new Error('Failed to save settings');
  }

  // Process API
  async getProcesses(): Promise<Process[]> {
    const response = await fetch(`${this.baseUrl}/processes`);
    if (!response.ok) throw new Error('Failed to load processes');
    return response.json();
  }

  async startProcess(id: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/processes/${id}/start`, {
      method: 'POST'
    });
    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to start process: ${error}`);
    }
  }

  async stopProcess(id: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/processes/${id}/stop`, {
      method: 'POST'
    });
    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to stop process: ${error}`);
    }
  }

  async removeProcess(id: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/processes/${id}`, {
      method: 'DELETE'
    });
    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to remove process: ${error}`);
    }
  }

  // Template API
  async getTemplates(): Promise<ProcessTemplate[]> {
    const response = await fetch(`${this.baseUrl}/templates`);
    if (!response.ok) throw new Error('Failed to load templates');
    return response.json();
  }

  async instantiateTemplate(
    templateId: string,
    processId: string,
    values: Record<string, any>
  ): Promise<void> {
    const response = await fetch(`${this.baseUrl}/templates/${templateId}/instantiate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        process_id: processId,
        values
      })
    });
    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to instantiate template: ${error}`);
    }
  }

  async createProcess(process: {
    id: string;
    command: string;
    args: string[];
    env: Record<string, string>;
    cwd: string | null;
  }): Promise<any> {
    const response = await fetch(`${this.baseUrl}/processes`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(process)
    });
    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to create process: ${error}`);
    }
    return response.json();
  }
}

export const apiClient = new ApiClient();