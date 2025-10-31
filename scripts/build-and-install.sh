#!/usr/bin/env bash

# Vantage MCP - Build and Install Script
# このスクリプトは以下を実行します:
# 1. UI (Vue) をビルド
# 2. Rustプロジェクトをweb featureでリリースビルド
# 3. ローカルからcargo installでインストール

set -e  # エラー時に即座に終了

# カラー設定
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ステータス表示関数
print_step() {
    echo -e "${BLUE}[$(date +'%H:%M:%S')]${NC} ${GREEN}▶${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# プロジェクトルートに移動
cd "$(dirname "$0")/.."
PROJECT_ROOT=$(pwd)

print_step "Vantage MCP Build & Install プロセスを開始"
echo "プロジェクトディレクトリ: $PROJECT_ROOT"
echo ""

# 1. UI のビルド
print_step "Step 1/3: UI (Vue) をビルド中..."
cd "$PROJECT_ROOT/ui/web"

if command -v bun &> /dev/null; then
    print_step "bunを使用してUIをビルド"
    bun install
    bun run build
elif command -v npm &> /dev/null; then
    print_warning "bunが見つかりません。npmを使用します"
    npm install
    npm run build
else
    print_error "bunまたはnpmがインストールされていません"
    exit 1
fi

if [ -d "dist" ]; then
    print_success "UIビルド完了 (dist ディレクトリ生成)"
else
    print_error "UIビルドに失敗しました"
    exit 1
fi

# 2. Rustプロジェクトのビルド
print_step "Step 2/3: Rustプロジェクトをビルド中..."
cd "$PROJECT_ROOT"

print_step "cargo cleanを実行（クリーンビルドを保証）"
cargo clean

print_step "リリースビルドを実行 (web feature付き)"
cargo build --release --features web

if [ -f "target/release/vantagemcp" ]; then
    print_success "Rustビルド完了"
else
    print_error "Rustビルドに失敗しました"
    exit 1
fi

# 3. cargo installでローカルインストール
print_step "Step 3/3: ローカルからcargo installでインストール中..."
cd "$PROJECT_ROOT"

# 既存のインストールを強制的に上書き
cargo install --path crates/vantage-mcp --force --features web

# インストール確認
if command -v vantagemcp &> /dev/null; then
    INSTALLED_PATH=$(which vantagemcp)
    print_success "インストール完了: $INSTALLED_PATH"

    # バージョン確認
    print_step "インストールされたバージョン情報:"
    vantagemcp --version || true
else
    print_error "インストールに失敗しました"
    exit 1
fi

echo ""
print_success "=== ビルド & インストール完了 ==="
echo ""
echo "使用方法:"
echo "  vantagemcp              # Vantage MCPサーバーを起動"
echo "  vantagemcp --no-open    # ブラウザを開かずに起動"
echo ""
echo "環境変数:"
echo "  VANTAGE_WEB_PORT=12703  # 別のポートで起動"
echo "  RUST_LOG=debug          # デバッグログを有効化"
echo ""
print_warning "注意: 既存のVantageプロセスが実行中の場合は、再起動が必要です"
echo ""

# 実行中のVantageプロセスをチェック
if pgrep -f vantagemcp > /dev/null; then
    print_warning "現在実行中のVantageプロセスが検出されました"
    echo "再起動する場合は以下を実行:"
    echo "  pkill -f vantagemcp"
    echo "  vantagemcp"
fi