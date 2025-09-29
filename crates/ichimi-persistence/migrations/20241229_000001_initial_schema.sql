-- Initial database schema for Ichimi Server

-- Process history table
CREATE TABLE IF NOT EXISTS process_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    process_id TEXT NOT NULL,
    name TEXT NOT NULL,
    command TEXT NOT NULL,
    args TEXT, -- JSON array
    env TEXT, -- JSON object
    cwd TEXT,
    started_at DATETIME NOT NULL,
    stopped_at DATETIME,
    exit_code INTEGER,
    error TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_process_history_process_id ON process_history(process_id);
CREATE INDEX idx_process_history_started_at ON process_history(started_at DESC);

-- Process metrics table
CREATE TABLE IF NOT EXISTS process_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    process_id TEXT NOT NULL,
    cpu_usage REAL,
    memory_usage INTEGER,
    disk_read INTEGER,
    disk_write INTEGER,
    network_rx INTEGER,
    network_tx INTEGER,
    recorded_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_process_metrics_process_id ON process_metrics(process_id);
CREATE INDEX idx_process_metrics_recorded_at ON process_metrics(recorded_at DESC);

-- Process output logs table
CREATE TABLE IF NOT EXISTS process_output (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    process_id TEXT NOT NULL,
    stream_type TEXT NOT NULL CHECK(stream_type IN ('stdout', 'stderr')),
    content TEXT NOT NULL,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_process_output_process_id ON process_output(process_id);
CREATE INDEX idx_process_output_timestamp ON process_output(timestamp DESC);

-- Clipboard table (for clipboard/paste board functionality)
CREATE TABLE IF NOT EXISTS clipboard (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL,
    content_type TEXT NOT NULL DEFAULT 'text',
    metadata TEXT, -- JSON object for additional metadata
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    accessed_at DATETIME,
    expires_at DATETIME
);

CREATE INDEX idx_clipboard_key ON clipboard(key);
CREATE INDEX idx_clipboard_updated_at ON clipboard(updated_at DESC);
CREATE INDEX idx_clipboard_expires_at ON clipboard(expires_at);

-- System events table
CREATE TABLE IF NOT EXISTS system_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    description TEXT NOT NULL,
    details TEXT, -- JSON object for additional details
    severity TEXT NOT NULL CHECK(severity IN ('info', 'warning', 'error')),
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_system_events_event_type ON system_events(event_type);
CREATE INDEX idx_system_events_timestamp ON system_events(timestamp DESC);
CREATE INDEX idx_system_events_severity ON system_events(severity);
