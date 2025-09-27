#!/usr/bin/env bun

/**
 * グレースフルシャットダウンを適切に処理するテスト用プロセス
 * SIGTERMを受信すると、クリーンアップ処理を実行してから正常終了する
 */

import { exit } from "process";

let isShuttingDown = false;
let taskCounter = 0;

// SIGTERM ハンドラー - グレースフルシャットダウン
process.on("SIGTERM", async () => {
  if (isShuttingDown) return;
  isShuttingDown = true;

  console.log("🛑 SIGTERM received - starting graceful shutdown...");
  console.log(`📊 Current task counter: ${taskCounter}`);

  try {
    // クリーンアップ処理をシミュレート
    for (let i = 1; i <= 3; i++) {
      console.log(`🧹 Cleanup step ${i}/3...`);
      await new Promise(resolve => setTimeout(resolve, 400));
    }

    // データ保存をシミュレート
    console.log("💾 Saving data...");
    await new Promise(resolve => setTimeout(resolve, 300));

    console.log("✅ Graceful shutdown completed successfully");
    exit(0);
  } catch (error) {
    console.error("❌ Error during graceful shutdown:", error);
    exit(1);
  }
});

// SIGINT ハンドラー（Ctrl+C）
process.on("SIGINT", () => {
  console.log("🔄 SIGINT received - delegating to SIGTERM handler");
  process.kill(process.pid, "SIGTERM");
});

// 予期しないエラーのハンドリング
process.on("uncaughtException", (error) => {
  console.error("💥 Uncaught exception:", error);
  process.exit(1);
});

process.on("unhandledRejection", (reason, promise) => {
  console.error("💥 Unhandled rejection at:", promise, "reason:", reason);
  process.exit(1);
});

console.log(`🚀 Graceful test process started with PID ${process.pid}`);
console.log("📝 Ready to handle SIGTERM gracefully");
console.log("⏰ Will perform cleanup operations before exiting");

// メインワークループ
const workInterval = setInterval(() => {
  if (!isShuttingDown) {
    taskCounter++;
    console.log(`⚙️  Working... (task #${taskCounter})`);

    // 長時間実行をシミュレート
    if (taskCounter % 5 === 0) {
      console.log(`📈 Milestone reached: ${taskCounter} tasks completed`);
    }
  }
}, 1500);

// シャットダウン時にインターバルをクリア
process.on("exit", () => {
  clearInterval(workInterval);
  console.log("🏁 Process exiting");
});