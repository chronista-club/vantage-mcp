#!/bin/bash

# Ichimi Development Server Script
# ファイル監視と自動再起動を含む開発サーバ

set -e

# 色付きメッセージ
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# クリーンアップ関数
cleanup() {
    log_warn "Shutting down development server..."
    if [ ! -z "$CARGO_WATCH_PID" ]; then
        kill $CARGO_WATCH_PID 2>/dev/null || true
        log_info "Stopped cargo-watch (PID: $CARGO_WATCH_PID)"
    fi
    if [ ! -z "$BUN_WATCH_PID" ]; then
        kill $BUN_WATCH_PID 2>/dev/null || true
        log_info "Stopped bun watch (PID: $BUN_WATCH_PID)"
    fi
    # 既存のichimiプロセスを停止
    pkill -f 'cargo run.*ichimi' 2>/dev/null || true
    pkill -f 'ichimi.*web-only' 2>/dev/null || true
    log_success "Development server shutdown complete"
}

# シグナルハンドラを設定
trap cleanup EXIT INT TERM

log_info "Starting Ichimi Development Server..."

# 既存のプロセスを停止
log_info "Stopping existing processes..."
pkill -f 'cargo run.*ichimi' 2>/dev/null || true
pkill -f 'cargo-watch' 2>/dev/null || true
sleep 1

# Webアセットのビルドとウォッチを開始
log_info "Starting web asset watch..."
if [ -d "ui/web" ]; then
    cd ui/web
    bun run build --watch &
    BUN_WATCH_PID=$!
    cd ../..
    log_success "Started bun watch (PID: $BUN_WATCH_PID)"
else
    log_warn "ui/web directory not found, skipping web asset watch"
fi

# cargo-watchを使用してRustコードの監視と自動再起動
log_info "Starting Rust code watch..."
RUST_LOG=info cargo watch \
    --watch crates/ichimi/src \
    --watch crates/ichimi/Cargo.toml \
    --clear \
    --exec 'run --bin ichimi -- --web-only' &
CARGO_WATCH_PID=$!

log_success "Started cargo-watch (PID: $CARGO_WATCH_PID)"
log_info "Development server is running!"
log_info "Rust code changes: crates/ichimi/src/**"
if [ ! -z "$BUN_WATCH_PID" ]; then
    log_info "Web asset changes: ui/web/src/**"
fi
log_info "Press Ctrl+C to stop the development server"

# プロセスを待機
wait