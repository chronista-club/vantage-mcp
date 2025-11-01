-- テンプレート用インデックス

-- 名前でユニーク
DEFINE INDEX template_name_idx ON TABLE template COLUMNS name UNIQUE OVERWRITE;

-- カテゴリで検索
DEFINE INDEX template_category_idx ON TABLE template COLUMNS category OVERWRITE;

-- 作成日時で並び替え
DEFINE INDEX template_created_at_idx ON TABLE template COLUMNS created_at OVERWRITE;

-- 使用回数で並び替え
DEFINE INDEX template_use_count_idx ON TABLE template COLUMNS use_count OVERWRITE;
