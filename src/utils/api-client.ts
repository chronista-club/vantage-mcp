/**
 * Ichimi Server API Client
 * Web API経由でプロセス管理を行うためのクライアント実装
 */

export interface ProcessCreateRequest {
  id: string;
  name: string;
  command: string;
  args: string[];
  env: Record<string, string>;
  cwd?: string;
  auto_start_on_restore?: boolean;
}

export interface ProcessStopRequest {
  id: string;
  grace_period_ms?: number;
}

export interface ProcessInfo {
  id: string;
  command: string;
  args: string[];
  env: Record<string, string>;
  cwd?: string;
  state: ProcessState;
  auto_start_on_restore: boolean;
}

export type ProcessState =
  | { NotStarted: null }
  | { Running: { pid: number; started_at: string } }
  | { Stopped: { exit_code?: number; stopped_at: string } }
  | { Failed: { error: string; failed_at: string } };

export interface ProcessLogsResponse {
  logs: string[];
  total_lines: number;
}

export interface ApiResponse<T> {
  data?: T;
  error?: string;
  success: boolean;
}

export class IchimiApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = "http://localhost:12701") {
    this.baseUrl = baseUrl.replace(/\/$/, ""); // 末尾のスラッシュを削除
  }

  /**
   * サーバーが起動しているかチェック
   */
  async checkHealth(): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/api/status`);
      return response.ok;
    } catch {
      return false;
    }
  }

  /**
   * プロセスを作成
   */
  async createProcess(request: ProcessCreateRequest): Promise<ApiResponse<void>> {
    try {
      const response = await fetch(`${this.baseUrl}/api/processes`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        const errorText = await response.text();
        return {
          success: false,
          error: `HTTP ${response.status}: ${errorText}`,
        };
      }

      return { success: true };
    } catch (error) {
      return {
        success: false,
        error: `Network error: ${error}`,
      };
    }
  }

  /**
   * プロセスを開始
   */
  async startProcess(id: string): Promise<ApiResponse<{ pid: number }>> {
    try {
      const response = await fetch(`${this.baseUrl}/api/processes/${id}/start`, {
        method: "POST",
      });

      if (!response.ok) {
        const errorText = await response.text();
        return {
          success: false,
          error: `HTTP ${response.status}: ${errorText}`,
        };
      }

      const text = await response.text();

      // Extract PID from the message "Process 'id' started with PID 12345"
      const pidMatch = text.match(/PID (\d+)/);
      if (!pidMatch) {
        return {
          success: false,
          error: `Could not extract PID from response: ${text}`,
        };
      }

      const pid = parseInt(pidMatch[1], 10);
      return {
        success: true,
        data: { pid },
      };
    } catch (error) {
      return {
        success: false,
        error: `Network error: ${error}`,
      };
    }
  }

  /**
   * プロセスを停止（グレースフルシャットダウン対応）
   */
  async stopProcess(request: ProcessStopRequest): Promise<ApiResponse<void>> {
    try {
      const response = await fetch(`${this.baseUrl}/api/processes/${request.id}/stop`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        const errorText = await response.text();
        return {
          success: false,
          error: `HTTP ${response.status}: ${errorText}`,
        };
      }

      // 204 No Content or 200 OK is expected
      return { success: true };
    } catch (error) {
      return {
        success: false,
        error: `Network error: ${error}`,
      };
    }
  }

  /**
   * プロセス状態を取得
   */
  async getProcessStatus(id: string): Promise<ApiResponse<ProcessInfo>> {
    try {
      const response = await fetch(`${this.baseUrl}/api/processes/${id}`);

      if (!response.ok) {
        const errorText = await response.text();
        return {
          success: false,
          error: `HTTP ${response.status}: ${errorText}`,
        };
      }

      const data = await response.json();
      // Extract the info field from the response
      return {
        success: true,
        data: data.info,
      };
    } catch (error) {
      return {
        success: false,
        error: `Network error: ${error}`,
      };
    }
  }

  /**
   * プロセスのログを取得
   */
  async getProcessLogs(id: string, maxLines: number = 50): Promise<ApiResponse<ProcessLogsResponse>> {
    try {
      const response = await fetch(`${this.baseUrl}/api/processes/${id}/logs?max_lines=${maxLines}`);

      if (!response.ok) {
        const errorText = await response.text();
        return {
          success: false,
          error: `HTTP ${response.status}: ${errorText}`,
        };
      }

      const logs = await response.json();
      return {
        success: true,
        data: {
          logs,
          total_lines: logs.length
        },
      };
    } catch (error) {
      return {
        success: false,
        error: `Network error: ${error}`,
      };
    }
  }

  /**
   * プロセスを削除
   */
  async removeProcess(id: string): Promise<ApiResponse<void>> {
    try {
      const response = await fetch(`${this.baseUrl}/api/processes/${id}`, {
        method: "DELETE",
      });

      if (!response.ok) {
        const errorText = await response.text();
        return {
          success: false,
          error: `HTTP ${response.status}: ${errorText}`,
        };
      }

      return { success: true };
    } catch (error) {
      return {
        success: false,
        error: `Network error: ${error}`,
      };
    }
  }

  /**
   * 全プロセス一覧を取得
   */
  async listProcesses(): Promise<ApiResponse<ProcessInfo[]>> {
    try {
      const response = await fetch(`${this.baseUrl}/api/processes`);

      if (!response.ok) {
        const errorText = await response.text();
        return {
          success: false,
          error: `HTTP ${response.status}: ${errorText}`,
        };
      }

      const data = await response.json();
      return {
        success: true,
        data,
      };
    } catch (error) {
      return {
        success: false,
        error: `Network error: ${error}`,
      };
    }
  }
}