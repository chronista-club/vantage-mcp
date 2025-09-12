-- Ichimi Server Database Export
-- Generated at: 2025-09-06 17:36:53.393601 UTC

USE NS ichimi DB main;

CREATE process CONTENT {"process_id":"test-completion","command":"echo","args":["Process completed"],"env":{},"cwd":null,"auto_start_on_restore":false};
CREATE process CONTENT {"process_id":"test-export-1","command":"sleep","args":["3600"],"env":{},"cwd":null,"auto_start_on_restore":true};
CREATE process CONTENT {"process_id":"test-export-2","command":"echo","args":["Hello from test-export-2"],"env":{},"cwd":null,"auto_start_on_restore":false};
