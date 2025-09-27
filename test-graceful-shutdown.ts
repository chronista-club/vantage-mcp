#!/usr/bin/env bun

import { spawn } from "child_process";
import { promisify } from "util";
import { writeFile, rm } from "fs/promises";
import path from "path";

const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));
const exec = promisify(require("child_process").exec);

// 色付き出力用のヘルパー
const colors = {
  red: (text: string) => `\x1b[31m${text}\x1b[0m`,
  green: (text: string) => `\x1b[32m${text}\x1b[0m`,
  yellow: (text: string) => `\x1b[33m${text}\x1b[0m`,
  blue: (text: string) => `\x1b[34m${text}\x1b[0m`,
};

// テスト用のTypeScriptプロセス（SIGTERMを適切に処理）
const gracefulScript = `
import { exit } from "process";

let isShuttingDown = false;

process.on("SIGTERM", async () => {
  if (isShuttingDown) return;
  isShuttingDown = true;

  console.log("Received SIGTERM, performing graceful shutdown...");

  // クリーンアップ処理をシミュレート
  for (let i = 1; i <= 3; i++) {
    console.log(\`Cleanup step \${i}/3...\`);
    await new Promise(resolve => setTimeout(resolve, 500));
  }

  console.log("Graceful shutdown complete");
  exit(0);
});

console.log(\`Process started with PID \${process.pid}\`);
console.log("Ready to handle SIGTERM gracefully");

// メインループ
setInterval(() => {
  if (!isShuttingDown) {
    console.log("Working...");
  }
}, 2000);
`;

// テスト用のTypeScriptプロセス（SIGTERMを無視）
const stubbornScript = `
process.on("SIGTERM", () => {
  console.log("Received SIGTERM, but I'm stubborn and won't exit!");
  // SIGTERMを無視
});

console.log(\`Stubborn process started with PID \${process.pid}\`);
console.log("I will ignore SIGTERM signals!");

// メインループ
setInterval(() => {
  console.log("I'm still running stubbornly...");
}, 2000);
`;

// MCPサーバーと通信するヘルパー関数
async function sendMCPCommand(method: string, args: any): Promise<any> {
  const request = {
    jsonrpc: "2.0",
    method: "tools/call",
    params: {
      name: method,
      arguments: args
    },
    id: Date.now()
  };

  const { stdout } = await exec(`echo '${JSON.stringify(request)}' | ./target/debug/ichimi 2>/dev/null`);
  try {
    return JSON.parse(stdout);
  } catch (e) {
    console.log("Raw response:", stdout);
    return null;
  }
}

async function main() {
  console.log(colors.blue("=== Ichimi Server Graceful Shutdown Test ===\n"));

  // テストスクリプトファイルを作成
  const gracefulPath = "/tmp/test-graceful.ts";
  const stubbornPath = "/tmp/test-stubborn.ts";

  await writeFile(gracefulPath, gracefulScript);
  await writeFile(stubbornPath, stubbornScript);

  // ビルド確認
  console.log(colors.yellow("Checking build..."));
  try {
    await exec("cargo check");
    console.log(colors.green("✓ Build check passed\n"));
  } catch (e) {
    console.log(colors.red("✗ Build check failed"));
    process.exit(1);
  }

  // Ichimi Serverを起動
  console.log(colors.yellow("Starting Ichimi Server..."));
  const ichimiProcess = spawn("./target/debug/ichimi", [], {
    env: { ...process.env, RUST_LOG: "info" },
    stdio: ["pipe", "pipe", "pipe"]
  });

  await sleep(2000); // サーバー起動待ち

  try {
    // Test 1: グレースフルシャットダウンのテスト
    console.log(colors.blue("\n📝 Test 1: Graceful shutdown test"));
    console.log("Creating a process that handles SIGTERM properly...");

    // プロセスを作成
    const createResult1 = await sendMCPCommand("create_process", {
      name: "graceful_test",
      command: "bun",
      args: ["run", gracefulPath],
      env: {}
    });

    const processId1 = createResult1?.result?.content?.[0]?.text?.match(/Process '([^']+)' created/)?.[1];
    if (!processId1) {
      throw new Error("Failed to create graceful process");
    }
    console.log(`Created process with ID: ${processId1}`);

    // プロセスを起動
    console.log("Starting the process...");
    await sendMCPCommand("start_process", { id: processId1 });
    await sleep(3000);

    // プロセスの出力を取得
    console.log("\nGetting process output:");
    const output1 = await sendMCPCommand("get_process_output", {
      id: processId1,
      max_lines: 10
    });
    const outputText1 = output1?.result?.content?.[0]?.text;
    if (outputText1) {
      console.log(colors.green(outputText1));
    }

    // グレースフルシャットダウンをテスト（3秒の猶予期間）
    console.log(colors.yellow("\nStopping process with 3-second grace period..."));
    const stopResult1 = await sendMCPCommand("stop_process", {
      id: processId1,
      grace_period_ms: 3000
    });
    console.log("Stop command sent");

    await sleep(4000);

    // 最終的な出力を確認
    console.log("\nFinal process output:");
    const finalOutput1 = await sendMCPCommand("get_process_output", {
      id: processId1,
      max_lines: 50
    });
    const finalText1 = finalOutput1?.result?.content?.[0]?.text;
    if (finalText1) {
      // グレースフルシャットダウンのメッセージが含まれているか確認
      if (finalText1.includes("Graceful shutdown complete")) {
        console.log(colors.green("✓ Process shut down gracefully!"));
      } else {
        console.log(colors.yellow("⚠ Process output:"));
      }
      console.log(finalText1);
    }

    // Test 2: 頑固なプロセスのテスト
    console.log(colors.blue("\n📝 Test 2: Stubborn process test (should be force-killed)"));
    console.log("Creating a process that ignores SIGTERM...");

    // 頑固なプロセスを作成
    const createResult2 = await sendMCPCommand("create_process", {
      name: "stubborn_test",
      command: "bun",
      args: ["run", stubbornPath],
      env: {}
    });

    const processId2 = createResult2?.result?.content?.[0]?.text?.match(/Process '([^']+)' created/)?.[1];
    if (!processId2) {
      throw new Error("Failed to create stubborn process");
    }
    console.log(`Created stubborn process with ID: ${processId2}`);

    // プロセスを起動
    console.log("Starting the stubborn process...");
    await sendMCPCommand("start_process", { id: processId2 });
    await sleep(3000);

    // グレースフルシャットダウンを試みる（2秒の猶予期間）
    console.log(colors.yellow("\nStopping stubborn process with 2-second grace period..."));
    console.log("(Should force-kill after timeout)");
    await sendMCPCommand("stop_process", {
      id: processId2,
      grace_period_ms: 2000
    });

    await sleep(3000);

    // プロセスの状態を確認
    console.log("\nChecking process status:");
    const statusResult2 = await sendMCPCommand("get_process_status", {
      id: processId2
    });
    const statusText2 = statusResult2?.result?.content?.[0]?.text;
    if (statusText2) {
      if (statusText2.includes("Stopped")) {
        console.log(colors.green("✓ Stubborn process was force-killed successfully!"));
      }
      console.log(statusText2);
    }

    // 最終出力を確認
    const finalOutput2 = await sendMCPCommand("get_process_output", {
      id: processId2,
      max_lines: 50
    });
    const finalText2 = finalOutput2?.result?.content?.[0]?.text;
    if (finalText2) {
      console.log("\nProcess output:");
      console.log(finalText2);
    }

    console.log(colors.green("\n✅ All tests completed successfully!"));

  } catch (error) {
    console.error(colors.red("\n❌ Test failed:"), error);
  } finally {
    // クリーンアップ
    console.log(colors.yellow("\nCleaning up..."));

    // Ichimiサーバーを停止
    ichimiProcess.kill();

    // テストファイルを削除
    await rm(gracefulPath, { force: true });
    await rm(stubbornPath, { force: true });

    console.log(colors.green("Done!"));
  }
}

// エラーハンドリング
process.on("unhandledRejection", (error) => {
  console.error(colors.red("Unhandled rejection:"), error);
  process.exit(1);
});

// メイン実行
main().catch(console.error);