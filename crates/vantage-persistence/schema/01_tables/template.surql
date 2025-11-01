-- プロセステンプレートテーブル
DEFINE TABLE template SCHEMAFULL OVERWRITE;

-- 基本フィールド
DEFINE FIELD name ON TABLE template TYPE string OVERWRITE
  ASSERT $value != NONE AND string::len($value) >= 1;

DEFINE FIELD description ON TABLE template TYPE option<string> OVERWRITE;

DEFINE FIELD category ON TABLE template TYPE string OVERWRITE
  ASSERT $value IN ["database", "web_server", "build_tool", "script", "other"]
  DEFAULT "other";

DEFINE FIELD tags ON TABLE template TYPE array<string> OVERWRITE
  DEFAULT [];

-- プロセス設定
DEFINE FIELD command ON TABLE template TYPE string OVERWRITE
  ASSERT $value != NONE AND string::len($value) >= 1;

DEFINE FIELD args ON TABLE template TYPE array<string> OVERWRITE
  DEFAULT [];

DEFINE FIELD env ON TABLE template TYPE object OVERWRITE
  DEFAULT {};

DEFINE FIELD cwd ON TABLE template TYPE option<string> OVERWRITE;

-- メタデータ
DEFINE FIELD created_at ON TABLE template TYPE datetime OVERWRITE
  DEFAULT time::now();

DEFINE FIELD updated_at ON TABLE template TYPE datetime OVERWRITE
  DEFAULT time::now()
  VALUE time::now();

DEFINE FIELD created_by ON TABLE template TYPE option<string> OVERWRITE;

-- 使用統計
DEFINE FIELD use_count ON TABLE template TYPE int OVERWRITE
  DEFAULT 0;

DEFINE FIELD last_used_at ON TABLE template TYPE option<datetime> OVERWRITE;
