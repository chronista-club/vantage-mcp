import axios from 'axios';
import type { AxiosInstance } from 'axios';
import type { 
  ProcessInfo, 
  ProcessTemplate, 
  Settings, 
  ClipboardItem, 
  ClipboardHistoryResponse 
} from '@/types';

class ApiClient {
  private client: AxiosInstance;

  constructor() {
    this.client = axios.create({
      baseURL: '/api',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    // Response interceptor for error handling
    this.client.interceptors.response.use(
      response => response,
      error => {
        const message = error.response?.data?.error || error.message || 'An error occurred';
        return Promise.reject(new Error(message));
      }
    );
  }

  // Settings API
  async getSettings(): Promise<Settings> {
    const { data } = await this.client.get<Settings>('/settings');
    return data;
  }

  async updateSettings(settings: Settings): Promise<void> {
    await this.client.put('/settings', settings);
  }

  // Process API
  async getProcesses(): Promise<ProcessInfo[]> {
    const { data } = await this.client.get<ProcessInfo[]>('/processes');
    return data;
  }

  async createProcess(process: {
    id: string;
    command: string;
    args: string[];
    env?: Record<string, string>;
    cwd?: string;
  }): Promise<ProcessInfo> {
    const { data } = await this.client.post<ProcessInfo>('/processes', process);
    return data;
  }

  async startProcess(id: string): Promise<void> {
    await this.client.post(`/processes/${encodeURIComponent(id)}/start`);
  }

  async stopProcess(id: string): Promise<void> {
    await this.client.post(`/processes/${encodeURIComponent(id)}/stop`);
  }

  async removeProcess(id: string): Promise<void> {
    await this.client.delete(`/processes/${encodeURIComponent(id)}`);
  }

  async getProcessOutput(id: string): Promise<{ stdout: string; stderr: string }> {
    const { data } = await this.client.get(`/processes/${encodeURIComponent(id)}/output`);
    return data;
  }

  // Template API
  async getTemplates(): Promise<ProcessTemplate[]> {
    const { data } = await this.client.get<ProcessTemplate[]>('/templates');
    return data;
  }

  async instantiateTemplate(
    templateId: string,
    processId: string,
    values: Record<string, any>
  ): Promise<ProcessInfo> {
    const { data } = await this.client.post<ProcessInfo>(
      `/templates/${encodeURIComponent(templateId)}/instantiate`,
      {
        process_id: processId,
        values
      }
    );
    return data;
  }

  // Clipboard API
  async getClipboard(): Promise<ClipboardItem | null> {
    try {
      const { data } = await this.client.get<ClipboardItem>('/clipboard');
      return data;
    } catch (error: any) {
      if (error.response?.status === 404) {
        return null;
      }
      throw error;
    }
  }

  async getClipboardHistory(limit?: number): Promise<ClipboardHistoryResponse> {
    const { data } = await this.client.get<ClipboardHistoryResponse>('/clipboard/history', {
      params: { limit }
    });
    return data;
  }

  async setClipboardText(content: string, tags: string[] = []): Promise<ClipboardItem> {
    const { data } = await this.client.post<ClipboardItem>('/clipboard/text', { 
      content, 
      tags 
    });
    return data;
  }

  async setClipboardFile(
    content: string, 
    filename: string, 
    tags: string[] = []
  ): Promise<ClipboardItem> {
    const { data } = await this.client.post<ClipboardItem>('/clipboard/file', { 
      content, 
      filename, 
      tags 
    });
    return data;
  }

  async deleteClipboardItem(id: string): Promise<void> {
    await this.client.delete(`/clipboard/${encodeURIComponent(id)}`);
  }

  async searchClipboard(query: string, limit?: number): Promise<ClipboardHistoryResponse> {
    const { data } = await this.client.get<ClipboardHistoryResponse>('/clipboard/search', {
      params: { query, limit }
    });
    return data;
  }

  async clearClipboard(): Promise<void> {
    await this.client.delete('/clipboard');
  }

  // Test processes
  async addTestProcesses(): Promise<ProcessInfo[]> {
    const { data } = await this.client.post<ProcessInfo[]>('/processes/test');
    return data;
  }
}

export const apiClient = new ApiClient();
export default apiClient;