export interface Process {
  id: string;
  command: string;
  args: string[];
  env: Record<string, string>;
  cwd?: string;
  state: ProcessState;
  pid?: number;
  cpu_usage?: number;
  memory_usage?: number;
  auto_start_on_restore: boolean;
  created_at: string;
  started_at?: string;
  stopped_at?: string;
  exit_code?: number;
  error_message?: string;
}

export type ProcessState = 'not_started' | 'running' | 'stopped' | 'failed';

export interface ProcessConfig {
  id: string;
  command: string;
  args?: string[];
  env?: Record<string, string>;
  cwd?: string;
  auto_start_on_restore?: boolean;
}

export interface ProcessOutput {
  lines: string[];
  has_more: boolean;
}

export interface ProcessStatus {
  process: Process;
  output: ProcessOutput;
}

export interface StatusResponse {
  version: string;
  process_count: number;
  running_count: number;
  uptime_seconds: number;
}