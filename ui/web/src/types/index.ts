// Process state types matching Rust enum structure
export type ProcessState = 
  | 'NotStarted'
  | { Running: { pid: number; started_at: string } }
  | { Stopped: { exit_code?: number; stopped_at: string } }
  | { Failed: { error: string; failed_at: string } };

// Process types
export interface Process {
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
  color_mode: 'light' | 'dark';
  auto_refresh: boolean;
  refresh_interval: number;
}