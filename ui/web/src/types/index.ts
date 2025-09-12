// Process state types matching Rust enum structure
export type ProcessState = 
  | 'NotStarted'
  | { Running: { pid: number; started_at: string } }
  | { Stopped: { exit_code?: number; stopped_at: string } }
  | { Failed: { error: string; failed_at: string } };

// Process types
export interface ProcessInfo {
  id: string;
  command: string;
  args: string[];
  cwd?: string;
  state: ProcessState;
  env?: Record<string, string>;
  auto_start_on_create?: boolean;
  auto_start_on_restore?: boolean;
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

// Settings types
export interface Settings {
  theme: 'light' | 'dark';
  auto_refresh: boolean;
  refresh_interval: number;
}

// Clipboard types
export interface ClipboardItem {
  id: string;
  content: string;
  filename?: string;
  created_at: string;
  updated_at: string;
  content_type: string;
  tags: string[];
}

export interface ClipboardHistoryResponse {
  total_count: number;
  items: ClipboardItem[];
}

// API Response types
export interface ApiResponse<T> {
  data: T;
  error?: string;
}

// Helper functions for state checking
export function isRunning(state: ProcessState): boolean {
  return typeof state === 'object' && 'Running' in state;
}

export function isStopped(state: ProcessState): boolean {
  return typeof state === 'object' && 'Stopped' in state;
}

export function isFailed(state: ProcessState): boolean {
  return typeof state === 'object' && 'Failed' in state;
}

export function isNotStarted(state: ProcessState): boolean {
  return state === 'NotStarted';
}

export function getStateLabel(state: ProcessState): string {
  if (isRunning(state)) return 'Running';
  if (isStopped(state)) return 'Stopped';
  if (isFailed(state)) return 'Failed';
  if (isNotStarted(state)) return 'Not Started';
  return 'Unknown';
}

export function getStateColor(state: ProcessState): string {
  if (isRunning(state)) return 'green';
  if (isStopped(state)) return 'yellow';
  if (isFailed(state)) return 'red';
  if (isNotStarted(state)) return 'secondary';
  return 'gray';
}