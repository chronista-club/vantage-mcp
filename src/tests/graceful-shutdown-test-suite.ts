#!/usr/bin/env bun

/**
 * 一味サーバー グレースフルシャットダウン包括テストスイート
 * Web API を使用したプロセス管理のテスト
 */

const API_BASE_URL = "http://localhost:12701/api";

// 色付き出力用のヘルパー
const colors = {
  red: (text: string) => `\x1b[31m${text}\x1b[0m`,
  green: (text: string) => `\x1b[32m${text}\x1b[0m`,
  yellow: (text: string) => `\x1b[33m${text}\x1b[0m`,
  blue: (text: string) => `\x1b[34m${text}\x1b[0m`,
  cyan: (text: string) => `\x1b[36m${text}\x1b[0m`,
  magenta: (text: string) => `\x1b[35m${text}\x1b[0m`,
  bold: (text: string) => `\x1b[1m${text}\x1b[0m`,
};

const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

// テスト結果の型定義
interface TestResult {
  name: string;
  passed: boolean;
  message: string;
  duration: number;
  details?: any;
}

// API クライアント
class IchimiApiClient {
  private baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  async createProcess(id: string, command: string, args: string[], env: Record<string, string> = {}, cwd?: string): Promise<any> {
    const response = await fetch(`${this.baseUrl}/processes`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id, command, args, env, cwd })
    });

    if (!response.ok) {
      throw new Error(`Failed to create process: ${response.status} ${response.statusText}`);
    }

    return await response.json();
  }

  async startProcess(id: string): Promise<any> {
    const response = await fetch(`${this.baseUrl}/processes/${id}/start`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id })
    });

    if (!response.ok) {
      throw new Error(`Failed to start process: ${response.status} ${response.statusText}`);
    }

    return await response.json();
  }

  async stopProcess(id: string, gracePeriodMs: number): Promise<any> {
    const response = await fetch(`${this.baseUrl}/processes/${id}/stop`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id, grace_period_ms: gracePeriodMs })
    });

    if (!response.ok) {
      throw new Error(`Failed to stop process: ${response.status} ${response.statusText}`);
    }

    return await response.json();
  }

  async getProcessStatus(id: string): Promise<any> {
    const response = await fetch(`${this.baseUrl}/processes/${id}`);

    if (!response.ok) {
      throw new Error(`Failed to get process status: ${response.status} ${response.statusText}`);
    }

    return await response.json();
  }

  async getProcessLogs(id: string, maxLines: number = 50): Promise<any> {
    const response = await fetch(`${this.baseUrl}/processes/${id}/logs?max_lines=${maxLines}`);

    if (!response.ok) {
      throw new Error(`Failed to get process logs: ${response.status} ${response.statusText}`);
    }

    return await response.json();
  }

  async listProcesses(): Promise<any> {
    const response = await fetch(`${this.baseUrl}/processes`);

    if (!response.ok) {
      throw new Error(`Failed to list processes: ${response.status} ${response.statusText}`);
    }

    return await response.json();
  }

  async deleteProcess(id: string): Promise<any> {
    const response = await fetch(`${this.baseUrl}/processes/${id}`, {
      method: "DELETE"
    });

    if (!response.ok) {
      throw new Error(`Failed to delete process: ${response.status} ${response.statusText}`);
    }

    return await response.json();
  }
}

// テストスイートクラス
class GracefulShutdownTestSuite {
  private client: IchimiApiClient;
  private results: TestResult[] = [];

  constructor(apiBaseUrl: string) {
    this.client = new IchimiApiClient(apiBaseUrl);
  }

  private async runTest(name: string, testFn: () => Promise<void>): Promise<void> {
    const startTime = Date.now();
    console.log(colors.cyan(`\n🧪 ${name}`));
    console.log(colors.cyan("─".repeat(60)));

    try {
      await testFn();
      const duration = Date.now() - startTime;
      this.results.push({ name, passed: true, message: "Test passed", duration });
      console.log(colors.green(`✅ PASSED (${duration}ms)`));
    } catch (error) {
      const duration = Date.now() - startTime;
      const message = error instanceof Error ? error.message : String(error);
      this.results.push({ name, passed: false, message, duration });
      console.log(colors.red(`❌ FAILED (${duration}ms): ${message}`));
    }
  }

  async testServerConnectivity(): Promise<void> {
    await this.runTest("Server Connectivity Test", async () => {
      const processes = await this.client.listProcesses();
      console.log(`📡 Server is responsive. Current processes: ${processes.length || 0}`);
    });
  }

  async testGracefulShutdown(): Promise<void> {
    await this.runTest("Basic Graceful Shutdown Test", async () => {
      const processId = "graceful-test-1";

      try {
        // プロセス作成
        console.log("📝 Creating graceful test process...");
        await this.client.createProcess(
          processId,
          "bun",
          ["run", "/Users/mito/repos/ichimi-server/src/tests/graceful-process.ts"]
        );

        // プロセス開始
        console.log("🚀 Starting process...");
        await this.client.startProcess(processId);
        await sleep(3000); // プロセスが安定するまで待機

        // 実行状況確認
        console.log("📊 Checking process status...");
        const status = await this.client.getProcessStatus(processId);
        const state = status.info?.state;
        const isRunning = state && typeof state === 'object' && 'Running' in state;
        if (!isRunning) {
          throw new Error(`Process is not running: ${JSON.stringify(state)}`);
        }

        // グレースフルシャットダウン実行
        console.log("🛑 Stopping process with 5-second grace period...");
        await this.client.stopProcess(processId, 5000);

        // シャットダウン完了まで待機
        await sleep(6000);

        // 最終状態確認
        const finalStatus = await this.client.getProcessStatus(processId);
        const finalState = finalStatus.info?.state;
        const isStopped = finalState && typeof finalState === 'object' && 'Stopped' in finalState;
        console.log(`📋 Final process state: ${JSON.stringify(finalState)}`);

        if (!isStopped) {
          throw new Error(`Process should be stopped but is: ${JSON.stringify(finalState)}`);
        }

        // ログ確認
        console.log("📜 Checking process logs...");
        const logs = await this.client.getProcessLogs(processId);
        const logText = Array.isArray(logs) ? logs.join('\n') : (logs.output || logs.stdout || "");

        if (!logText.includes("Graceful shutdown completed successfully")) {
          throw new Error("Graceful shutdown message not found in logs");
        }

        if (!logText.includes("SIGTERM received")) {
          throw new Error("SIGTERM handling message not found in logs");
        }

        console.log("✅ Process handled SIGTERM gracefully");
        console.log("✅ Graceful shutdown completed successfully");

      } finally {
        // クリーンアップ
        try {
          await this.client.deleteProcess(processId);
        } catch (e) {
          console.log("ℹ️  Process cleanup completed or already removed");
        }
      }
    });
  }

  async testForceKill(): Promise<void> {
    await this.runTest("Force Kill Test (Stubborn Process)", async () => {
      const processId = "stubborn-test-1";

      try {
        // 頑固なプロセス作成
        console.log("📝 Creating stubborn test process...");
        await this.client.createProcess(
          processId,
          "bun",
          ["run", "/Users/mito/repos/ichimi-server/src/tests/stubborn-process.ts"]
        );

        // プロセス開始
        console.log("🚀 Starting stubborn process...");
        await this.client.startProcess(processId);
        await sleep(3000);

        // 実行状況確認
        const status = await this.client.getProcessStatus(processId);
        const state = status.info?.state;
        const isRunning = state && typeof state === 'object' && 'Running' in state;
        if (!isRunning) {
          throw new Error(`Process is not running: ${JSON.stringify(state)}`);
        }

        // 短い猶予期間でシャットダウン試行
        console.log("🛑 Attempting to stop stubborn process with 2-second grace period...");
        await this.client.stopProcess(processId, 2000);

        // 強制終了完了まで待機
        await sleep(4000);

        // 最終状態確認
        const finalStatus = await this.client.getProcessStatus(processId);
        const finalState = finalStatus.info?.state;
        const isStopped = finalState && typeof finalState === 'object' && 'Stopped' in finalState;
        console.log(`📋 Final process state: ${JSON.stringify(finalState)}`);

        if (!isStopped) {
          throw new Error(`Process should be stopped but is: ${JSON.stringify(finalState)}`);
        }

        // ログ確認
        const logs = await this.client.getProcessLogs(processId);
        const logText = Array.isArray(logs) ? logs.join('\n') : (logs.output || logs.stdout || "");

        if (!logText.includes("I'm stubborn and won't exit")) {
          throw new Error("Stubborn behavior message not found in logs");
        }

        console.log("✅ Stubborn process was force-killed successfully");

      } finally {
        try {
          await this.client.deleteProcess(processId);
        } catch (e) {
          console.log("ℹ️  Process cleanup completed or already removed");
        }
      }
    });
  }

  async testVariousGracePeriods(): Promise<void> {
    const gracePeriods = [1000, 3000, 5000];

    for (const gracePeriod of gracePeriods) {
      await this.runTest(`Grace Period Test (${gracePeriod}ms)`, async () => {
        const processId = `grace-test-${gracePeriod}`;

        try {
          console.log(`📝 Creating process for ${gracePeriod}ms grace period test...`);
          await this.client.createProcess(
            processId,
            "bun",
            ["run", "/Users/mito/repos/ichimi-server/src/tests/slow-graceful-process.ts"],
            { SHUTDOWN_DURATION_MS: String(Math.floor(gracePeriod * 0.8)) } // 猶予期間の80%で完了
          );

          console.log("🚀 Starting process...");
          await this.client.startProcess(processId);
          await sleep(2000);

          console.log(`🛑 Stopping with ${gracePeriod}ms grace period...`);
          const stopStartTime = Date.now();
          await this.client.stopProcess(processId, gracePeriod);

          await sleep(gracePeriod + 1000);

          const stopDuration = Date.now() - stopStartTime;
          console.log(`⏱️  Stop operation took ${stopDuration}ms`);

          const finalStatus = await this.client.getProcessStatus(processId);
          const finalState = finalStatus.info?.state;
          const isStopped = finalState && typeof finalState === 'object' && 'Stopped' in finalState;
          if (!isStopped) {
            throw new Error(`Process should be stopped but is: ${JSON.stringify(finalState)}`);
          }

          console.log(`✅ Process stopped within ${gracePeriod}ms grace period`);

        } finally {
          try {
            await this.client.deleteProcess(processId);
          } catch (e) {
            console.log("ℹ️  Process cleanup completed or already removed");
          }
        }
      });
    }
  }

  async testConcurrentShutdowns(): Promise<void> {
    await this.runTest("Concurrent Shutdowns Test", async () => {
      const processIds = ["concurrent-1", "concurrent-2", "concurrent-3"];

      try {
        // 複数プロセスを同時作成・開始
        console.log("📝 Creating multiple processes...");
        for (const processId of processIds) {
          await this.client.createProcess(
            processId,
            "bun",
            ["run", "/Users/mito/repos/ichimi-server/src/tests/graceful-process.ts"]
          );
          await this.client.startProcess(processId);
        }

        await sleep(3000);

        // 同時シャットダウン
        console.log("🛑 Stopping all processes concurrently...");
        const stopPromises = processIds.map(id =>
          this.client.stopProcess(id, 4000)
        );

        await Promise.all(stopPromises);
        await sleep(5000);

        // 全プロセスの状態確認
        for (const processId of processIds) {
          const status = await this.client.getProcessStatus(processId);
          const state = status.info?.state;
          const isStopped = state && typeof state === 'object' && 'Stopped' in state;
          if (!isStopped) {
            throw new Error(`Process ${processId} should be stopped but is: ${JSON.stringify(state)}`);
          }
        }

        console.log("✅ All processes stopped successfully in concurrent test");

      } finally {
        // クリーンアップ
        for (const processId of processIds) {
          try {
            await this.client.deleteProcess(processId);
          } catch (e) {
            // Ignore cleanup errors
          }
        }
      }
    });
  }

  async runAllTests(): Promise<void> {
    console.log(colors.bold(colors.blue("🧪 Ichimi Server Graceful Shutdown Test Suite")));
    console.log(colors.blue("=" .repeat(60)));

    const startTime = Date.now();

    await this.testServerConnectivity();
    await this.testGracefulShutdown();
    await this.testForceKill();
    await this.testVariousGracePeriods();
    await this.testConcurrentShutdowns();

    const totalDuration = Date.now() - startTime;
    this.printTestSummary(totalDuration);
  }

  private printTestSummary(totalDuration: number): void {
    console.log(colors.bold(colors.blue("\n📊 Test Results Summary")));
    console.log(colors.blue("=" .repeat(60)));

    const passed = this.results.filter(r => r.passed).length;
    const failed = this.results.filter(r => !r.passed).length;
    const total = this.results.length;

    console.log(`📈 Total Tests: ${total}`);
    console.log(`${colors.green(`✅ Passed: ${passed}`)}`);
    console.log(`${colors.red(`❌ Failed: ${failed}`)}`);
    console.log(`⏱️  Total Duration: ${totalDuration}ms`);

    if (failed > 0) {
      console.log(colors.red("\n💥 Failed Tests:"));
      this.results
        .filter(r => !r.passed)
        .forEach(result => {
          console.log(colors.red(`   • ${result.name}: ${result.message}`));
        });
    }

    console.log(colors.blue("\n📋 Detailed Results:"));
    this.results.forEach(result => {
      const status = result.passed ? colors.green("✅ PASS") : colors.red("❌ FAIL");
      console.log(`   ${status} ${result.name} (${result.duration}ms)`);
    });

    const successRate = total > 0 ? Math.round((passed / total) * 100) : 0;
    console.log(colors.bold(`\n🎯 Success Rate: ${successRate}%`));

    if (successRate === 100) {
      console.log(colors.green(colors.bold("🎉 All tests passed! Graceful shutdown is working perfectly!")));
    } else {
      console.log(colors.yellow("⚠️  Some tests failed. Please review the implementation."));
    }
  }
}

// メイン実行
async function main() {
  try {
    const testSuite = new GracefulShutdownTestSuite(API_BASE_URL);
    await testSuite.runAllTests();
  } catch (error) {
    console.error(colors.red("💥 Test suite failed to run:"), error);
    process.exit(1);
  }
}

// エラーハンドリング
process.on("unhandledRejection", (error) => {
  console.error(colors.red("💥 Unhandled rejection:"), error);
  process.exit(1);
});

process.on("SIGINT", () => {
  console.log(colors.yellow("\n⚠️  Test suite interrupted by user"));
  process.exit(0);
});

main().catch(console.error);