-- テンプレート用インデックス

-- 名前でユニーク
DEFINE INDEX IF NOT EXISTS template_name_idx ON TABLE template COLUMNS name UNIQUE;

-- カテゴリで検索
DEFINE INDEX IF NOT EXISTS template_category_idx ON TABLE template COLUMNS category;

-- 作成日時で並び替え
DEFINE INDEX IF NOT EXISTS template_created_at_idx ON TABLE template COLUMNS created_at;

-- 使用回数で並び替え
DEFINE INDEX IF NOT EXISTS template_use_count_idx ON TABLE template COLUMNS use_count;
