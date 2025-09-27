#!/usr/bin/env bun

/**
 * 時間のかかるグレースフルシャットダウンプロセス
 * 様々なgrace_period_msでのテストに使用
 */

import { exit } from "process";

let isShuttingDown = false;
let operationCounter = 0;

// 設定可能なシャットダウン時間（環境変数から取得）
const shutdownDurationMs = parseInt(process.env.SHUTDOWN_DURATION_MS || "2000");
const taskIntervalMs = parseInt(process.env.TASK_INTERVAL_MS || "800");

process.on("SIGTERM", async () => {
  if (isShuttingDown) return;
  isShuttingDown = true;

  console.log(`🐌 SIGTERM received - starting SLOW graceful shutdown...`);
  console.log(`⏳ Shutdown will take approximately ${shutdownDurationMs}ms`);
  console.log(`📊 Operations completed before shutdown: ${operationCounter}`);

  try {
    // 時間のかかるクリーンアップ処理
    const steps = Math.ceil(shutdownDurationMs / 400);
    for (let i = 1; i <= steps; i++) {
      console.log(`🔄 Slow cleanup step ${i}/${steps}...`);
      await new Promise(resolve => setTimeout(resolve, 400));
    }

    console.log("✅ Slow graceful shutdown completed");
    exit(0);
  } catch (error) {
    console.error("❌ Error during slow graceful shutdown:", error);
    exit(1);
  }
});

console.log(`🐌 Slow graceful test process started with PID ${process.pid}`);
console.log(`⏳ Configured shutdown duration: ${shutdownDurationMs}ms`);
console.log(`⚙️  Task interval: ${taskIntervalMs}ms`);

// メインワークループ
const workInterval = setInterval(() => {
  if (!isShuttingDown) {
    operationCounter++;
    console.log(`🔄 Slow operation ${operationCounter}...`);
  }
}, taskIntervalMs);

process.on("exit", () => {
  clearInterval(workInterval);
  console.log("🏁 Slow process exiting");
});