-- Ichimi Server Database Export
-- Generated at: 2025-09-02 03:45:21.004702 UTC

USE NS ichimi DB main;

CREATE process CONTENT {"process_id":"test-signal-2","command":"sleep","args":["600"],"env":{},"cwd":null,"auto_start_on_restore":false};
CREATE process CONTENT {"process_id":"test-signal-1","command":"echo","args":["Testing signal handler"],"env":{},"cwd":null,"auto_start_on_restore":false};
