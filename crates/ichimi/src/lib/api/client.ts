import type { Process, ProcessConfig, ProcessOutput, ProcessStatus, StatusResponse } from '$lib/types';

class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = '') {
    this.baseUrl = baseUrl;
  }

  private async request<T>(path: string, options?: RequestInit): Promise<T> {
    const url = `${this.baseUrl}/api${path}`;
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || `HTTP ${response.status}`);
    }

    if (response.status === 204) {
      return {} as T;
    }

    return response.json();
  }

  async getStatus(): Promise<StatusResponse> {
    return this.request<StatusResponse>('/status');
  }

  async listProcesses(): Promise<Process[]> {
    return this.request<Process[]>('/processes');
  }

  async getProcess(id: string): Promise<ProcessStatus> {
    return this.request<ProcessStatus>(`/processes/${id}`);
  }

  async createProcess(config: ProcessConfig): Promise<void> {
    await this.request('/processes', {
      method: 'POST',
      body: JSON.stringify(config),
    });
  }

  async startProcess(id: string): Promise<void> {
    await this.request(`/processes/${id}/start`, {
      method: 'POST',
    });
  }

  async stopProcess(id: string): Promise<void> {
    await this.request(`/processes/${id}/stop`, {
      method: 'POST',
    });
  }

  async restartProcess(id: string): Promise<void> {
    await this.request(`/processes/${id}/restart`, {
      method: 'POST',
    });
  }

  async removeProcess(id: string): Promise<void> {
    await this.request(`/processes/${id}`, {
      method: 'DELETE',
    });
  }

  async getProcessOutput(id: string): Promise<ProcessOutput> {
    return this.request<ProcessOutput>(`/processes/${id}/output`);
  }

  async exportProcesses(): Promise<void> {
    await this.request('/export', {
      method: 'POST',
    });
  }

  async importProcesses(): Promise<void> {
    await this.request('/import', {
      method: 'POST',
    });
  }
}

export const api = new ApiClient();