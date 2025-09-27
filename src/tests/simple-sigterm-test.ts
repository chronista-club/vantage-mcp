#!/usr/bin/env bun

/**
 * Simple SIGTERM Test for debugging
 * シンプルなSIGTERMテストでデバッグを行う
 */

import { IchimiApiClient } from "../utils/api-client.ts";
import chalk from "chalk";

const apiClient = new IchimiApiClient("http://localhost:12701");

async function simpleSigTermTest() {
  console.log(chalk.blue("🧪 Simple SIGTERM Test"));
  console.log(chalk.gray("=" .repeat(40)));

  const processId = `debug-${Date.now()}`;

  try {
    // 1. プロセス作成
    console.log(chalk.cyan("1. Creating test process..."));
    const createResponse = await apiClient.createProcess({
      id: processId,
      name: "Debug SIGTERM Test",
      command: "bun",
      args: ["run", "/tmp/sleep-test.ts"],
      env: {},
    });

    if (!createResponse.success) {
      throw new Error(`Failed to create process: ${createResponse.error}`);
    }
    console.log(chalk.green(`✓ Process '${processId}' created`));

    // 2. プロセス開始
    console.log(chalk.cyan("2. Starting process..."));
    const startResponse = await apiClient.startProcess(processId);
    if (!startResponse.success) {
      throw new Error(`Failed to start process: ${startResponse.error}`);
    }
    console.log(chalk.green(`✓ Process started with PID ${startResponse.data!.pid}`));

    // 3. プロセスが実際に動いていることを確認
    await new Promise(resolve => setTimeout(resolve, 2000));
    const statusResponse = await apiClient.getProcessStatus(processId);
    if (!statusResponse.success) {
      throw new Error(`Failed to get status: ${statusResponse.error}`);
    }

    const state = statusResponse.data!.state;
    if (!("Running" in state)) {
      throw new Error(`Process is not running: ${JSON.stringify(state)}`);
    }
    console.log(chalk.green(`✓ Process confirmed running with PID ${state.Running.pid}`));

    // 4. プロセス停止（SIGTERM）
    console.log(chalk.cyan("3. Stopping process with SIGTERM..."));
    const stopResponse = await apiClient.stopProcess({
      id: processId,
      grace_period_ms: 5000,
    });

    if (!stopResponse.success) {
      throw new Error(`Failed to stop process: ${stopResponse.error}`);
    }
    console.log(chalk.green("✓ Stop command sent"));

    // 5. 停止完了を待機
    console.log(chalk.cyan("4. Waiting for process to stop..."));
    let attempts = 0;
    while (attempts < 20) {
      await new Promise(resolve => setTimeout(resolve, 500));
      const checkResponse = await apiClient.getProcessStatus(processId);
      if (checkResponse.success) {
        const state = checkResponse.data!.state;
        if ("Stopped" in state || "Failed" in state) {
          console.log(chalk.green(`✓ Process stopped: ${JSON.stringify(state)}`));
          break;
        }
        console.log(chalk.gray(`  Still running... attempt ${attempts + 1}/20`));
      }
      attempts++;
    }

    if (attempts >= 20) {
      console.log(chalk.red("✗ Process did not stop within timeout"));
    }

    // 6. ログ確認
    console.log(chalk.cyan("5. Checking logs..."));
    await new Promise(resolve => setTimeout(resolve, 1000));
    const logsResponse = await apiClient.getProcessLogs(processId, 100);
    if (logsResponse.success) {
      console.log(chalk.blue("📋 Process logs:"));
      const logs = logsResponse.data!.logs;
      logs.forEach((log, index) => {
        if (log.includes("SIGTERM") || log.includes("graceful") || log.includes("Received")) {
          console.log(chalk.yellow(`  ${index + 1}: ${log}`));
        } else {
          console.log(chalk.gray(`  ${index + 1}: ${log}`));
        }
      });

      // SIGTERM関連のメッセージを検索
      const sigtermLogs = logs.filter(log =>
        log.includes("SIGTERM") ||
        log.includes("graceful") ||
        log.includes("Received")
      );

      if (sigtermLogs.length > 0) {
        console.log(chalk.green(`✓ Found ${sigtermLogs.length} SIGTERM-related log entries`));
      } else {
        console.log(chalk.red("✗ No SIGTERM-related log entries found"));
      }
    } else {
      console.log(chalk.red(`Failed to get logs: ${logsResponse.error}`));
    }

  } catch (error) {
    console.log(chalk.red(`❌ Test failed: ${error}`));
  } finally {
    // Cleanup
    try {
      await apiClient.removeProcess(processId);
      console.log(chalk.gray(`🧹 Cleaned up process '${processId}'`));
    } catch {
      // Ignore cleanup errors
    }
  }
}

if (import.meta.main) {
  simpleSigTermTest().catch(console.error);
}