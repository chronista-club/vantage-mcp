-- プロセステンプレートテーブル
DEFINE TABLE IF NOT EXISTS template SCHEMAFULL;

-- 基本フィールド
DEFINE FIELD IF NOT EXISTS name ON TABLE template TYPE string
  ASSERT $value != NONE AND string::len($value) >= 1;

DEFINE FIELD IF NOT EXISTS description ON TABLE template TYPE option<string>;

DEFINE FIELD IF NOT EXISTS category ON TABLE template TYPE string
  ASSERT $value IN ["database", "web_server", "build_tool", "script", "other"]
  DEFAULT "other";

DEFINE FIELD IF NOT EXISTS tags ON TABLE template TYPE array<string>
  DEFAULT [];

-- プロセス設定
DEFINE FIELD IF NOT EXISTS command ON TABLE template TYPE string
  ASSERT $value != NONE AND string::len($value) >= 1;

DEFINE FIELD IF NOT EXISTS args ON TABLE template TYPE array<string>
  DEFAULT [];

DEFINE FIELD IF NOT EXISTS env ON TABLE template TYPE object
  DEFAULT {};

DEFINE FIELD IF NOT EXISTS cwd ON TABLE template TYPE option<string>;

-- メタデータ
DEFINE FIELD IF NOT EXISTS created_at ON TABLE template TYPE option<string>;

DEFINE FIELD IF NOT EXISTS updated_at ON TABLE template TYPE option<string>;

DEFINE FIELD IF NOT EXISTS created_by ON TABLE template TYPE option<string>;

-- 使用統計
DEFINE FIELD IF NOT EXISTS use_count ON TABLE template TYPE int
  DEFAULT 0;

DEFINE FIELD IF NOT EXISTS last_used_at ON TABLE template TYPE option<string>;
