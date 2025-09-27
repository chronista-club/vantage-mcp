/**
 * Test Utilities for Ichimi Server Graceful Shutdown Tests
 * テスト実行のためのユーティリティ関数群
 */

import chalk from "chalk";

// テスト結果の型定義
export interface TestResult {
  name: string;
  passed: boolean;
  duration: number;
  error?: string;
  details?: Record<string, any>;
}

export interface TestSuite {
  name: string;
  tests: TestResult[];
  setup?: () => Promise<void>;
  teardown?: () => Promise<void>;
}

export interface TestReport {
  suites: TestSuite[];
  totalTests: number;
  passedTests: number;
  failedTests: number;
  totalDuration: number;
  startTime: Date;
  endTime: Date;
}

/**
 * テスト実行器
 */
export class TestRunner {
  private suites: TestSuite[] = [];
  private verbose: boolean;

  constructor(verbose: boolean = true) {
    this.verbose = verbose;
  }

  /**
   * テストスイートを追加
   */
  addSuite(suite: TestSuite): void {
    this.suites.push(suite);
  }

  /**
   * 全テストを実行
   */
  async runAll(): Promise<TestReport> {
    const startTime = new Date();

    this.log(chalk.blue.bold("\n🧪 Ichimi Server Graceful Shutdown Test Suite"));
    this.log(chalk.gray("=" .repeat(60)));

    let totalTests = 0;
    let passedTests = 0;
    let failedTests = 0;

    for (const suite of this.suites) {
      this.log(chalk.cyan.bold(`\n📋 ${suite.name}`));
      this.log(chalk.gray("-".repeat(40)));

      // Setup
      if (suite.setup) {
        try {
          await suite.setup();
          this.log(chalk.gray("✓ Setup completed"));
        } catch (error) {
          this.log(chalk.red(`✗ Setup failed: ${error}`));
          continue;
        }
      }

      // Tests
      for (const test of suite.tests) {
        totalTests++;
        if (test.passed) {
          passedTests++;
          this.log(chalk.green(`  ✓ ${test.name} (${test.duration}ms)`));
        } else {
          failedTests++;
          this.log(chalk.red(`  ✗ ${test.name} (${test.duration}ms)`));
          if (test.error) {
            this.log(chalk.red(`    Error: ${test.error}`));
          }
        }
      }

      // Teardown
      if (suite.teardown) {
        try {
          await suite.teardown();
          this.log(chalk.gray("✓ Teardown completed"));
        } catch (error) {
          this.log(chalk.yellow(`⚠ Teardown warning: ${error}`));
        }
      }
    }

    const endTime = new Date();
    const totalDuration = endTime.getTime() - startTime.getTime();

    // Summary
    this.log(chalk.blue.bold("\n📊 Test Summary"));
    this.log(chalk.gray("=" .repeat(60)));
    this.log(`Total Tests: ${totalTests}`);
    this.log(chalk.green(`Passed: ${passedTests}`));
    this.log(chalk.red(`Failed: ${failedTests}`));
    this.log(`Duration: ${totalDuration}ms`);
    this.log(`Success Rate: ${totalTests > 0 ? ((passedTests / totalTests) * 100).toFixed(1) : 0}%`);

    if (failedTests === 0) {
      this.log(chalk.green.bold("\n🎉 All tests passed!"));
    } else {
      this.log(chalk.red.bold(`\n❌ ${failedTests} test(s) failed.`));
    }

    return {
      suites: this.suites,
      totalTests,
      passedTests,
      failedTests,
      totalDuration,
      startTime,
      endTime,
    };
  }

  private log(message: string): void {
    if (this.verbose) {
      console.log(message);
    }
  }
}

/**
 * 非同期関数の実行時間を測定
 */
export async function measureTime<T>(fn: () => Promise<T>): Promise<{ result: T; duration: number }> {
  const start = Date.now();
  const result = await fn();
  const duration = Date.now() - start;
  return { result, duration };
}

/**
 * 指定時間待機
 */
export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * 条件が満たされるまで待機（ポーリング）
 */
export async function waitUntil(
  condition: () => Promise<boolean>,
  timeoutMs: number = 10000,
  intervalMs: number = 100
): Promise<boolean> {
  const start = Date.now();

  while (Date.now() - start < timeoutMs) {
    if (await condition()) {
      return true;
    }
    await sleep(intervalMs);
  }

  return false;
}

/**
 * テスト用のランダムID生成
 */
export function generateTestId(prefix: string = "test"): string {
  const timestamp = Date.now();
  const random = Math.random().toString(36).substring(2, 8);
  return `${prefix}-${timestamp}-${random}`;
}

/**
 * ログエントリの検索
 */
export function searchLogs(logs: string[], patterns: string[]): {
  found: boolean;
  matches: { pattern: string; line: string; index: number }[];
} {
  const matches: { pattern: string; line: string; index: number }[] = [];

  for (const pattern of patterns) {
    const regex = new RegExp(pattern, 'i');

    for (let i = 0; i < logs.length; i++) {
      const line = logs[i];
      if (regex.test(line)) {
        matches.push({ pattern, line, index: i });
      }
    }
  }

  return {
    found: matches.length === patterns.length,
    matches,
  };
}

/**
 * タイムスタンプ付きログ出力
 */
export function logWithTimestamp(message: string, level: "info" | "warn" | "error" = "info"): void {
  const timestamp = new Date().toISOString();
  const prefix = chalk.gray(`[${timestamp}]`);

  switch (level) {
    case "info":
      console.log(`${prefix} ${message}`);
      break;
    case "warn":
      console.log(`${prefix} ${chalk.yellow(message)}`);
      break;
    case "error":
      console.log(`${prefix} ${chalk.red(message)}`);
      break;
  }
}

/**
 * エラーハンドリング付きのテスト実行
 */
export async function runTest(
  name: string,
  testFn: () => Promise<void>,
  timeoutMs: number = 30000
): Promise<TestResult> {
  const startTime = Date.now();

  try {
    // タイムアウト付きでテスト実行
    await Promise.race([
      testFn(),
      new Promise<never>((_, reject) =>
        setTimeout(() => reject(new Error(`Test timeout after ${timeoutMs}ms`)), timeoutMs)
      )
    ]);

    return {
      name,
      passed: true,
      duration: Date.now() - startTime,
    };
  } catch (error) {
    return {
      name,
      passed: false,
      duration: Date.now() - startTime,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

/**
 * テスト結果をJSONで出力
 */
export function exportTestReport(report: TestReport, filePath?: string): void {
  const reportData = {
    ...report,
    timestamp: new Date().toISOString(),
    environment: {
      platform: process.platform,
      node_version: process.version,
      bun_version: process.versions.bun,
    },
  };

  if (filePath) {
    Bun.write(filePath, JSON.stringify(reportData, null, 2));
    console.log(chalk.blue(`📄 Test report exported to: ${filePath}`));
  } else {
    console.log(chalk.blue("\n📄 Test Report (JSON):"));
    console.log(JSON.stringify(reportData, null, 2));
  }
}