#!/usr/bin/env bun

/**
 * Ichimi Server Graceful Shutdown Test Suite
 *
 * このテストスイートは以下のシナリオを検証します：
 * 1. グレースフルシャットダウン（SIGTERMによる正常終了）
 * 2. 強制終了（グレースピリオド後のSIGKILL）
 * 3. ログ記録の確認（適切なメッセージが記録されること）
 */

import { IchimiApiClient, type ProcessCreateRequest, type ProcessStopRequest } from "../utils/api-client.ts";
import {
  TestRunner,
  type TestSuite,
  runTest,
  measureTime,
  waitUntil,
  generateTestId,
  searchLogs,
  logWithTimestamp,
  exportTestReport,
  sleep,
} from "../utils/test-utils.ts";
import chalk from "chalk";

// テスト設定
const TEST_CONFIG = {
  SERVER_URL: "http://localhost:12701",
  SLEEP_TEST_SCRIPT: "/tmp/sleep-test.ts",
  HEALTH_CHECK_TIMEOUT: 5000,
  PROCESS_START_TIMEOUT: 3000,
  GRACEFUL_SHUTDOWN_TIMEOUT: 10000,
  FORCE_KILL_TIMEOUT: 15000,
  LOG_CHECK_DELAY: 1000,
};

// APIクライアント
const apiClient = new IchimiApiClient(TEST_CONFIG.SERVER_URL);

/**
 * サーバーヘルスチェック
 */
async function checkServerHealth(): Promise<void> {
  logWithTimestamp("Checking Ichimi Server health...", "info");

  const isHealthy = await waitUntil(
    () => apiClient.checkHealth(),
    TEST_CONFIG.HEALTH_CHECK_TIMEOUT,
    500
  );

  if (!isHealthy) {
    throw new Error(
      `Ichimi Server is not responding at ${TEST_CONFIG.SERVER_URL}. ` +
      "Please ensure the server is running with: cargo run --bin ichimi -- --web"
    );
  }

  logWithTimestamp("✓ Server health check passed", "info");
}

/**
 * テスト用プロセスの作成
 */
async function createTestProcess(id: string): Promise<void> {
  const request: ProcessCreateRequest = {
    id,
    name: `Graceful Shutdown Test Process - ${id}`,
    command: "bun",
    args: ["run", TEST_CONFIG.SLEEP_TEST_SCRIPT],
    env: {},
    auto_start_on_restore: false,
  };

  const response = await apiClient.createProcess(request);
  if (!response.success) {
    throw new Error(`Failed to create test process: ${response.error}`);
  }

  logWithTimestamp(`✓ Test process '${id}' created`, "info");
}

/**
 * プロセスの開始と検証
 */
async function startAndVerifyProcess(id: string): Promise<number> {
  const response = await apiClient.startProcess(id);
  if (!response.success || !response.data) {
    throw new Error(`Failed to start process: ${response.error}`);
  }

  const pid = response.data.pid;
  logWithTimestamp(`✓ Process '${id}' started with PID ${pid}`, "info");

  // プロセスが実際に開始されたことを確認
  await sleep(TEST_CONFIG.LOG_CHECK_DELAY);

  const statusResponse = await apiClient.getProcessStatus(id);
  if (!statusResponse.success || !statusResponse.data) {
    throw new Error(`Failed to get process status: ${statusResponse.error}`);
  }

  const state = statusResponse.data.state;
  if (!("Running" in state)) {
    throw new Error(`Process is not running. State: ${JSON.stringify(state)}`);
  }

  return pid;
}

/**
 * プロセスの停止と検証
 */
async function stopAndVerifyProcess(
  id: string,
  gracePeriodMs: number,
  expectGracefulShutdown: boolean
): Promise<{ duration: number; logs: string[] }> {
  const request: ProcessStopRequest = {
    id,
    grace_period_ms: gracePeriodMs,
  };

  logWithTimestamp(
    `Stopping process '${id}' with grace period ${gracePeriodMs}ms...`,
    "info"
  );

  const { duration } = await measureTime(async () => {
    const response = await apiClient.stopProcess(request);
    if (!response.success) {
      throw new Error(`Failed to stop process: ${response.error}`);
    }
  });

  // プロセスが停止したことを確認
  const stopped = await waitUntil(async () => {
    const statusResponse = await apiClient.getProcessStatus(id);
    if (!statusResponse.success) return false;

    const state = statusResponse.data!.state;
    return "Stopped" in state || "Failed" in state;
  }, TEST_CONFIG.GRACEFUL_SHUTDOWN_TIMEOUT);

  if (!stopped) {
    throw new Error("Process did not stop within timeout");
  }

  // ログを取得
  await sleep(TEST_CONFIG.LOG_CHECK_DELAY);
  const logsResponse = await apiClient.getProcessLogs(id, 100);
  if (!logsResponse.success) {
    throw new Error(`Failed to get logs: ${logsResponse.error}`);
  }

  const logs = logsResponse.data!.logs;

  logWithTimestamp(
    `✓ Process '${id}' stopped in ${duration}ms (expected graceful: ${expectGracefulShutdown})`,
    "info"
  );

  return { duration, logs };
}

/**
 * ログの内容を検証
 */
function verifyLogs(logs: string[], expectedPatterns: string[], testName: string): void {
  logWithTimestamp(`Verifying logs for ${testName}...`, "info");

  const { found, matches } = searchLogs(logs, expectedPatterns);

  if (!found) {
    const missingPatterns = expectedPatterns.filter(
      pattern => !matches.some(m => m.pattern === pattern)
    );

    logWithTimestamp("Available logs:", "info");
    logs.forEach((log, index) => {
      console.log(chalk.gray(`  ${index + 1}: ${log}`));
    });

    throw new Error(
      `Missing expected log patterns: ${missingPatterns.join(", ")}\n` +
      `Found matches: ${matches.map(m => `"${m.pattern}" in "${m.line}"`).join(", ")}`
    );
  }

  logWithTimestamp(
    `✓ All expected log patterns found for ${testName}`,
    "info"
  );

  // マッチした内容を表示
  matches.forEach(match => {
    logWithTimestamp(
      `  Pattern "${match.pattern}" matched: "${match.line}"`,
      "info"
    );
  });
}

/**
 * テストケース: グレースフルシャットダウン
 */
async function testGracefulShutdown(): Promise<void> {
  const processId = generateTestId("graceful");

  try {
    await createTestProcess(processId);
    const pid = await startAndVerifyProcess(processId);

    // 十分な猶予期間でプロセスを停止
    const { duration, logs } = await stopAndVerifyProcess(
      processId,
      5000, // 5秒の猶予期間
      true   // グレースフルシャットダウンを期待
    );

    // ログを検証
    const expectedPatterns = [
      "Received SIGTERM signal",
      "Starting graceful shutdown",
      "Cleanup complete, exiting gracefully",
    ];

    verifyLogs(logs, expectedPatterns, "Graceful Shutdown");

    // グレースフルシャットダウンは通常1秒程度で完了するはず
    if (duration > 3000) {
      logWithTimestamp(
        `Warning: Shutdown took ${duration}ms, which is longer than expected for graceful shutdown`,
        "warn"
      );
    }

    logWithTimestamp(`✓ Graceful shutdown completed in ${duration}ms`, "info");
  } finally {
    // クリーンアップ
    try {
      await apiClient.removeProcess(processId);
    } catch {
      // 削除エラーは無視
    }
  }
}

/**
 * テストケース: 強制終了（タイムアウト）
 */
async function testForceKill(): Promise<void> {
  const processId = generateTestId("force-kill");

  try {
    await createTestProcess(processId);
    await startAndVerifyProcess(processId);

    // 短い猶予期間でプロセスを停止（強制終了になることを期待）
    const { duration, logs } = await stopAndVerifyProcess(
      processId,
      500,  // 0.5秒の短い猶予期間
      false // 強制終了を期待
    );

    // ログを検証
    const expectedPatterns = [
      "Received SIGTERM signal",
      "Starting graceful shutdown",
    ];

    // 注意: 強制終了の場合、"Cleanup complete"のメッセージは出力されない可能性がある
    // SIGKILLによって強制的に終了されるため

    verifyLogs(logs, expectedPatterns, "Force Kill");

    logWithTimestamp(`✓ Force kill completed in ${duration}ms`, "info");
  } finally {
    // クリーンアップ
    try {
      await apiClient.removeProcess(processId);
    } catch {
      // 削除エラーは無視
    }
  }
}

/**
 * テストケース: 通常終了（自然な終了）
 */
async function testNormalCompletion(): Promise<void> {
  const processId = generateTestId("normal");

  try {
    await createTestProcess(processId);
    await startAndVerifyProcess(processId);

    // プロセスが自然に終了するまで待機（15秒後）
    logWithTimestamp("Waiting for process to complete naturally...", "info");

    const completed = await waitUntil(async () => {
      const statusResponse = await apiClient.getProcessStatus(processId);
      if (!statusResponse.success) return false;

      const state = statusResponse.data!.state;
      return "Stopped" in state && state.Stopped.exit_code === 0;
    }, 20000); // 20秒待機

    if (!completed) {
      throw new Error("Process did not complete naturally within timeout");
    }

    // ログを取得
    const logsResponse = await apiClient.getProcessLogs(processId, 100);
    if (!logsResponse.success) {
      throw new Error(`Failed to get logs: ${logsResponse.error}`);
    }

    const logs = logsResponse.data!.logs;

    // 通常終了のログを検証
    const expectedPatterns = [
      "Sleep complete! Process exiting normally",
    ];

    verifyLogs(logs, expectedPatterns, "Normal Completion");

    logWithTimestamp("✓ Process completed normally", "info");
  } finally {
    // クリーンアップ
    try {
      await apiClient.removeProcess(processId);
    } catch {
      // 削除エラーは無視
    }
  }
}

/**
 * メイン実行関数
 */
async function main(): Promise<void> {
  const runner = new TestRunner(true);

  // テストスイートを設定
  const gracefulShutdownSuite: TestSuite = {
    name: "Graceful Shutdown Tests",
    tests: [],
    setup: async () => {
      await checkServerHealth();
    },
  };

  // テストケースを追加
  gracefulShutdownSuite.tests.push(
    await runTest("Graceful Shutdown", testGracefulShutdown, 30000)
  );

  gracefulShutdownSuite.tests.push(
    await runTest("Force Kill", testForceKill, 30000)
  );

  gracefulShutdownSuite.tests.push(
    await runTest("Normal Completion", testNormalCompletion, 30000)
  );

  runner.addSuite(gracefulShutdownSuite);

  // テスト実行
  const report = await runner.runAll();

  // レポート出力
  const reportPath = `/Users/mito/repos/ichimi-server/test-report-${Date.now()}.json`;
  exportTestReport(report, reportPath);

  // 終了コード設定
  if (report.failedTests > 0) {
    process.exit(1);
  } else {
    process.exit(0);
  }
}

// スクリプト実行
if (import.meta.main) {
  main().catch(error => {
    console.error(chalk.red.bold("\n❌ Test execution failed:"));
    console.error(chalk.red(error.message));
    if (error.stack) {
      console.error(chalk.gray(error.stack));
    }
    process.exit(1);
  });
}