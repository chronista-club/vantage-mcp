# Ichimi Server プロジェクト概要

## プロジェクトの目的
Ichimi Server は Model Context Protocol (MCP) を介した Claude Code 用のプロセス管理サーバーです。プロセスをリソースとして扱い、Claude がプロセスの起動、停止、監視、および出力のキャプチャを可能にします。

## 技術スタック
- **言語**: Rust
- **バージョン**: 0.1.0-beta8
- **主要フレームワーク/ライブラリ**:
  - tokio (非同期ランタイム)
  - serde (シリアライゼーション)
  - rmcp (Model Context Protocol SDK)
  - SurrealDB (データベース、kv-rocksdb バックエンド)
  - axum (Webサーバー、オプショナル)
  - tracing (ロギング)
  - facet/kdl (設定ファイル形式)

## 主な機能
- プロセスのライフサイクル管理（作成、起動、停止、削除）
- リアルタイムログキャプチャ（stdout/stderr）
- SurrealDB による永続化
- KDL形式でのインポート/エクスポート
- Webダッシュボード（Alpine.js + Tabler UI）
- 自動バックアップ機能
- 学習エンジンによるスマート提案

## プロジェクト作者
Mako <mito@chronista.club>