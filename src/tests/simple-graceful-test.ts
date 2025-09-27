#!/usr/bin/env bun

/**
 * シンプルなグレースフルシャットダウンテスト
 * デバッグ用の詳細な情報を出力
 */

const API_BASE_URL = "http://localhost:12701/api";

const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

async function testBasicGracefulShutdown() {
  console.log("🧪 Simple Graceful Shutdown Test");
  console.log("=".repeat(50));

  try {
    // 1. プロセス作成
    console.log("\n1️⃣ Creating process...");
    const createResponse = await fetch(`${API_BASE_URL}/processes`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        id: "simple-test",
        command: "bun",
        args: ["run", "/Users/mito/repos/ichimi-server/src/tests/graceful-process.ts"]
      })
    });

    if (!createResponse.ok) {
      throw new Error(`Create failed: ${createResponse.status} ${createResponse.statusText}`);
    }

    const createResult = await createResponse.json();
    console.log("✅ Process created:", createResult);

    // 2. プロセス開始
    console.log("\n2️⃣ Starting process...");
    const startResponse = await fetch(`${API_BASE_URL}/processes/simple-test/start`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: "simple-test" })
    });

    if (!startResponse.ok) {
      throw new Error(`Start failed: ${startResponse.status} ${startResponse.statusText}`);
    }

    const startResult = await startResponse.json();
    console.log("✅ Process started:", startResult);

    // 3. 少し待つ
    console.log("\n3️⃣ Waiting for process to stabilize...");
    await sleep(3000);

    // 4. 状態確認
    console.log("\n4️⃣ Checking process status...");
    const statusResponse = await fetch(`${API_BASE_URL}/processes/simple-test`);
    const status = await statusResponse.json();
    console.log("📊 Process status:", JSON.stringify(status, null, 2));

    // 5. ログ確認（開始後）
    console.log("\n5️⃣ Checking initial logs...");
    const logsResponse = await fetch(`${API_BASE_URL}/processes/simple-test/logs?max_lines=10`);
    const logs = await logsResponse.json();
    console.log("📜 Initial logs:", logs);

    // 6. グレースフルシャットダウン
    console.log("\n6️⃣ Stopping process gracefully...");
    const stopResponse = await fetch(`${API_BASE_URL}/processes/simple-test/stop`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: "simple-test", grace_period_ms: 5000 })
    });

    if (!stopResponse.ok) {
      const errorText = await stopResponse.text();
      throw new Error(`Stop failed: ${stopResponse.status} ${stopResponse.statusText} - ${errorText}`);
    }

    const stopResult = await stopResponse.json();
    console.log("✅ Stop command sent:", stopResult);

    // 7. シャットダウン完了まで待機
    console.log("\n7️⃣ Waiting for shutdown to complete...");
    await sleep(6000);

    // 8. 最終状態確認
    console.log("\n8️⃣ Checking final status...");
    const finalStatusResponse = await fetch(`${API_BASE_URL}/processes/simple-test`);
    const finalStatus = await finalStatusResponse.json();
    console.log("📊 Final status:", JSON.stringify(finalStatus, null, 2));

    // 9. 最終ログ確認
    console.log("\n9️⃣ Checking final logs...");
    const finalLogsResponse = await fetch(`${API_BASE_URL}/processes/simple-test/logs?max_lines=50`);
    const finalLogs = await finalLogsResponse.json();
    console.log("📜 Final logs:", finalLogs);

    // 10. 分析
    console.log("\n🔍 Analysis:");
    const state = finalStatus.info?.state;
    const isStopped = state && typeof state === 'object' && 'Stopped' in state;
    const isRunning = state && typeof state === 'object' && 'Running' in state;

    console.log(`- Process state: ${JSON.stringify(state)}`);
    console.log(`- Is stopped: ${isStopped}`);
    console.log(`- Is running: ${isRunning}`);

    if (Array.isArray(finalLogs)) {
      const logText = finalLogs.join('\n');
      console.log(`- Contains SIGTERM message: ${logText.includes('SIGTERM received')}`);
      console.log(`- Contains graceful shutdown message: ${logText.includes('Graceful shutdown completed')}`);
    }

  } catch (error) {
    console.error("❌ Test failed:", error);
  } finally {
    // クリーンアップ
    try {
      console.log("\n🧹 Cleanup...");
      await fetch(`${API_BASE_URL}/processes/simple-test`, { method: "DELETE" });
      console.log("✅ Cleanup completed");
    } catch (e) {
      console.log("ℹ️  Cleanup failed or not needed");
    }
  }
}

testBasicGracefulShutdown().catch(console.error);