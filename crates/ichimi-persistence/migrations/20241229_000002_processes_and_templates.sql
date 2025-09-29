-- Processes table (current configuration)
CREATE TABLE IF NOT EXISTS processes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    process_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    command TEXT NOT NULL,
    args TEXT, -- JSON array
    env TEXT, -- JSON object
    cwd TEXT,
    state TEXT NOT NULL DEFAULT 'not_started',
    pid INTEGER,
    exit_code INTEGER,
    started_at DATETIME,
    stopped_at DATETIME,
    error TEXT,
    tags TEXT, -- JSON array
    auto_start_on_restore INTEGER DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_processes_process_id ON processes(process_id);
CREATE INDEX idx_processes_state ON processes(state);
CREATE INDEX idx_processes_updated_at ON processes(updated_at DESC);

-- Process templates table
CREATE TABLE IF NOT EXISTS process_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    template_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT,
    command TEXT NOT NULL,
    args TEXT, -- JSON array
    env TEXT, -- JSON object
    default_cwd TEXT,
    default_auto_start INTEGER DEFAULT 0,
    variables TEXT, -- JSON array of template variables
    tags TEXT, -- JSON array
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_process_templates_template_id ON process_templates(template_id);
CREATE INDEX idx_process_templates_category ON process_templates(category);
CREATE INDEX idx_process_templates_updated_at ON process_templates(updated_at DESC);

-- Settings table
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);