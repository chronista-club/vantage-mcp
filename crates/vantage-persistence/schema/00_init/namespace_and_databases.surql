-- ネームスペースとデータベースの初期化
--
-- このスクリプトは vantage ネームスペースと、
-- main（本番用）および test（テスト用）データベースを作成します。

-- ネームスペース vantage を定義（存在しない場合のみ作成）
DEFINE NAMESPACE IF NOT EXISTS vantage;

-- vantage ネームスペースに切り替え
USE NAMESPACE vantage;

-- データベース main を定義（アプリケーション用）
DEFINE DATABASE IF NOT EXISTS main;

-- データベース test を定義（テスト用）
DEFINE DATABASE IF NOT EXISTS test;
