#!/usr/bin/env bun
// release.ts - リリースプロセスのメインスクリプト

import type { ReleaseConfig } from "./types.ts";
import {
  parseVersion,
  formatVersion,
  formatTag,
  getCurrentVersion,
  updateCargoToml,
  updateCargoLock,
  runCargoTest,
  runCargoBuildRelease,
  getCurrentBranch,
  createCommit,
  createTag,
  pushToRemote,
  checkPrerequisites,
} from "./lib.ts";

// ============================================================
// ヘルパー関数
// ============================================================

/**
 * エラーメッセージを表示して終了する
 */
function exitWithError(message: string): never {
  console.error(`\n❌ エラー: ${message}\n`);
  process.exit(1);
}

/**
 * ユーザー確認を求める（yes/no）
 */
async function confirm(message: string): Promise<boolean> {
  const answer = prompt(`${message} (yes/no): `);
  return answer?.toLowerCase() === "yes" || answer?.toLowerCase() === "y";
}

/**
 * Cargo.tomlを元に戻す
 */
async function rollbackCargoToml(originalVersion: string): Promise<void> {
  console.log("🔄 Cargo.tomlをロールバックしています...");
  const content = await Bun.file("Cargo.toml").text();
  const restored = content.replace(
    /^version\s*=\s*"[^"]+"/m,
    `version = "${originalVersion}"`
  );
  await Bun.write("Cargo.toml", restored);
  await Bun.$`cargo build --quiet`;
}

// ============================================================
// メイン処理
// ============================================================

async function main() {
  console.log("\n📦 Vantage MCP リリースプロセス開始\n");

  // ============================================================
  // 1. コマンドライン引数のパース
  // ============================================================
  const args = process.argv.slice(2);
  if (args.length === 0) {
    console.log("使用方法: bun run release.ts <version>");
    console.log("例: bun run release.ts 0.1.0-beta21");
    process.exit(1);
  }

  const newVersion = parseVersion(args[0]);
  if (!newVersion) {
    exitWithError(`無効なバージョン形式: ${args[0]}`);
  }

  // ============================================================
  // 2. 事前チェック
  // ============================================================
  console.log("📋 事前チェック中...\n");

  const checkResults = await checkPrerequisites();
  const failedChecks = checkResults.filter((r) => !r.passed);

  if (failedChecks.length > 0) {
    console.log("以下のチェックが失敗しました:\n");
    for (const check of failedChecks) {
      console.log(`  ❌ ${check.error}`);
    }
    exitWithError("事前チェックに失敗しました");
  }

  console.log("  ✅ すべてのチェックをパスしました\n");

  // ============================================================
  // 3. バージョン情報の表示
  // ============================================================
  const currentVersion = await getCurrentVersion();
  const config: ReleaseConfig = {
    currentVersion,
    newVersion,
    tag: formatTag(newVersion),
  };

  console.log("📝 リリース情報:");
  console.log(`  現在のバージョン: ${formatVersion(currentVersion)}`);
  console.log(`  新しいバージョン: ${formatVersion(newVersion)}`);
  console.log(`  タグ: ${config.tag}\n`);

  const shouldProceed = await confirm("このバージョンでリリースを続行しますか？");
  if (!shouldProceed) {
    console.log("\n❌ リリースがキャンセルされました\n");
    process.exit(0);
  }

  // ============================================================
  // 4. Cargo.toml と Cargo.lock の更新
  // ============================================================
  console.log("\n🔧 Cargo.toml を更新中...");
  const originalVersion = formatVersion(currentVersion);

  try {
    await updateCargoToml(newVersion);
    console.log("  ✅ Cargo.toml を更新しました");

    console.log("\n🔧 Cargo.lock を更新中...");
    await updateCargoLock();
    console.log("  ✅ Cargo.lock を更新しました");
  } catch (error) {
    await rollbackCargoToml(originalVersion);
    exitWithError(`ファイル更新に失敗: ${error}`);
  }

  // ============================================================
  // 5. テストとビルド
  // ============================================================
  console.log("\n🧪 テストを実行中...");

  const testPassed = await runCargoTest();
  if (!testPassed) {
    await rollbackCargoToml(originalVersion);
    exitWithError("テストに失敗しました");
  }

  console.log("  ✅ すべてのテストが通りました");

  console.log("\n🔨 リリースビルドを実行中...");

  const buildPassed = await runCargoBuildRelease();
  if (!buildPassed) {
    await rollbackCargoToml(originalVersion);
    exitWithError("ビルドに失敗しました");
  }

  console.log("  ✅ ビルドが成功しました");

  // ============================================================
  // 6. Git コミット
  // ============================================================
  console.log("\n📦 Git コミットを作成中...");

  const commitMessage = `chore: bump version to ${config.tag}`;
  const filesToCommit = ["Cargo.toml", "Cargo.lock"];

  try {
    await createCommit(commitMessage, filesToCommit);
    console.log(`  ✅ コミット作成: "${commitMessage}"`);
  } catch (error) {
    await rollbackCargoToml(originalVersion);
    exitWithError(`コミット作成に失敗: ${error}`);
  }

  // ============================================================
  // 7. Git タグ
  // ============================================================
  console.log("\n🏷️  Git タグを作成中...");

  const tagMessage = `Release ${config.tag}`;

  try {
    await createTag(config.tag, tagMessage);
    console.log(`  ✅ タグ作成: ${config.tag}`);
  } catch (error) {
    // タグ作成失敗時はコミットを取り消し
    await Bun.$`git reset --hard HEAD~1`;
    await rollbackCargoToml(originalVersion);
    exitWithError(`タグ作成に失敗: ${error}`);
  }

  // ============================================================
  // 8. リモートへのプッシュ（確認付き）
  // ============================================================
  console.log("\n🚢 リモートへのプッシュ準備完了");
  console.log(`  ブランチ: ${await getCurrentBranch()}`);
  console.log(`  タグ: ${config.tag}\n`);

  const shouldPush = await confirm("リモートにプッシュしますか？");

  if (shouldPush) {
    try {
      const branch = await getCurrentBranch();
      await pushToRemote(branch, config.tag);
      console.log("\n  ✅ リモートにプッシュしました\n");
    } catch (error) {
      exitWithError(`プッシュに失敗: ${error}`);
    }

    // ============================================================
    // 9. 次のステップを表示
    // ============================================================
    console.log("🎉 リリースプロセスが完了しました！\n");
    console.log("次のステップ:");
    console.log(`  1. GitHubでリリースを作成:`);
    console.log(`     gh release create ${config.tag} \\`);
    console.log(`       --title "${config.tag} - タイトル" \\`);
    console.log(`       --notes-file release-notes.md \\`);
    console.log(`       --prerelease\n`);
    console.log(`  2. ユーザーにインストール方法を案内:`);
    console.log(`     cargo install --git https://github.com/chronista-club/vantage-mcp --tag ${config.tag} vantage-mcp\n`);
  } else {
    console.log("\n⚠️  プッシュがスキップされました");
    console.log("後でプッシュする場合:");
    console.log(`  git push origin ${await getCurrentBranch()}`);
    console.log(`  git push origin ${config.tag}\n`);
  }
}

// ============================================================
// エントリーポイント
// ============================================================
main().catch((error) => {
  console.error("\n❌ 予期しないエラーが発生しました:");
  console.error(error);
  process.exit(1);
});
