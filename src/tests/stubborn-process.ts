#!/usr/bin/env bun

/**
 * SIGTERMを無視する頑固なテスト用プロセス
 * 強制終了（SIGKILL）でのみ停止可能
 */

let requestCounter = 0;

// SIGTERM を無視
process.on("SIGTERM", () => {
  console.log("🙅 SIGTERM received, but I'm stubborn and won't exit!");
  console.log("💪 I'll keep working no matter what!");
  console.log("⚠️  Only SIGKILL can stop me now...");
});

// SIGINT も無視
process.on("SIGINT", () => {
  console.log("🙅 SIGINT received, but I'm ignoring it too!");
});

// プロセス情報
console.log(`😈 Stubborn test process started with PID ${process.pid}`);
console.log("🛡️  I will ignore SIGTERM and SIGINT signals!");
console.log("⚔️  Only SIGKILL can defeat me!");

// メインワークループ（頑固に動き続ける）
setInterval(() => {
  requestCounter++;
  console.log(`💀 Still running stubbornly... (request #${requestCounter})`);

  if (requestCounter % 3 === 0) {
    console.log("👹 I'm being extra stubborn today!");
  }

  if (requestCounter % 7 === 0) {
    console.log("🔥 You can't stop me! I'm unstoppable!");
  }
}, 1200);