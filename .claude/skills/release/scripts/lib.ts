// lib.ts - リリーススクリプトのユーティリティ関数

import type { Version, CheckResult } from "./types.ts";

// ============================================================
// Version Utils
// ============================================================

/**
 * バージョン文字列をパースする
 * @param str バージョン文字列（例: "0.1.0-beta20"）
 * @returns パースされたVersionオブジェクト、または失敗時はnull
 */
export function parseVersion(str: string): Version | null {
  const match = str.match(/^(\d+)\.(\d+)\.(\d+)(?:-(.+))?$/);
  if (!match) return null;

  return {
    major: parseInt(match[1], 10),
    minor: parseInt(match[2], 10),
    patch: parseInt(match[3], 10),
    prerelease: match[4],
  };
}

/**
 * Versionオブジェクトを文字列にフォーマットする
 * @param v Versionオブジェクト
 * @returns バージョン文字列（例: "0.1.0-beta20"）
 */
export function formatVersion(v: Version): string {
  const base = `${v.major}.${v.minor}.${v.patch}`;
  return v.prerelease ? `${base}-${v.prerelease}` : base;
}

/**
 * Versionオブジェクトをタグ形式にフォーマットする
 * @param v Versionオブジェクト
 * @returns タグ文字列（例: "v0.1.0-beta20"）
 */
export function formatTag(v: Version): string {
  return `v${formatVersion(v)}`;
}

// ============================================================
// Cargo Utils
// ============================================================

/**
 * Cargo.tomlから現在のバージョンを取得する
 * @returns 現在のVersionオブジェクト
 * @throws バージョンが見つからないか、無効な形式の場合
 */
export async function getCurrentVersion(): Promise<Version> {
  const content = await Bun.file("Cargo.toml").text();
  const match = content.match(/^version\s*=\s*"([^"]+)"/m);

  if (!match) {
    throw new Error("Cargo.tomlからバージョンを取得できません");
  }

  const version = parseVersion(match[1]);
  if (!version) {
    throw new Error(`無効なバージョン形式: ${match[1]}`);
  }

  return version;
}

/**
 * Cargo.tomlのバージョンを更新する
 * @param newVersion 新しいVersionオブジェクト
 */
export async function updateCargoToml(newVersion: Version): Promise<void> {
  const content = await Bun.file("Cargo.toml").text();
  const versionStr = formatVersion(newVersion);

  const updated = content.replace(
    /^version\s*=\s*"[^"]+"/m,
    `version = "${versionStr}"`
  );

  await Bun.write("Cargo.toml", updated);
}

/**
 * Cargo.lockを更新する（cargo buildを実行）
 */
export async function updateCargoLock(): Promise<void> {
  await Bun.$`cargo build --quiet`;
}

/**
 * cargo testを実行する
 * @returns テストが成功した場合はtrue、失敗した場合はfalse
 */
export async function runCargoTest(): Promise<boolean> {
  try {
    await Bun.$`cargo test --quiet`;
    return true;
  } catch {
    return false;
  }
}

/**
 * cargo build --releaseを実行する
 * @returns ビルドが成功した場合はtrue、失敗した場合はfalse
 */
export async function runCargoBuildRelease(): Promise<boolean> {
  try {
    await Bun.$`cargo build --release --quiet`;
    return true;
  } catch {
    return false;
  }
}

// ============================================================
// Git Utils
// ============================================================

/**
 * 現在のブランチ名を取得する
 * @returns ブランチ名
 */
export async function getCurrentBranch(): Promise<string> {
  return (await Bun.$`git branch --show-current`.text()).trim();
}

/**
 * 未コミットの変更があるかチェックする
 * @returns 未コミットの変更がある場合はtrue
 */
export async function hasUncommittedChanges(): Promise<boolean> {
  const status = await Bun.$`git status --porcelain`.text();
  return status.trim().length > 0;
}

/**
 * リモートと同期しているかチェックする
 * @returns 同期している場合はtrue
 */
export async function isSyncedWithRemote(): Promise<boolean> {
  try {
    await Bun.$`git fetch origin`;
    const local = await Bun.$`git rev-parse @`.text();
    const remote = await Bun.$`git rev-parse @{u}`.text();
    return local.trim() === remote.trim();
  } catch {
    // リモートブランチが存在しない場合はtrue（初回プッシュの場合）
    return true;
  }
}

/**
 * Gitコミットを作成する
 * @param message コミットメッセージ
 * @param files コミットするファイルのパス配列
 */
export async function createCommit(
  message: string,
  files: string[]
): Promise<void> {
  await Bun.$`git add ${files}`;
  await Bun.$`git commit -m ${message}`;
}

/**
 * Gitタグを作成する
 * @param tag タグ名
 * @param message タグメッセージ
 */
export async function createTag(tag: string, message: string): Promise<void> {
  await Bun.$`git tag -a ${tag} -m ${message}`;
}

/**
 * リモートにプッシュする
 * @param branch ブランチ名
 * @param tag タグ名
 */
export async function pushToRemote(
  branch: string,
  tag: string
): Promise<void> {
  await Bun.$`git push origin ${branch}`;
  await Bun.$`git push origin ${tag}`;
}

// ============================================================
// Pre-flight Checks
// ============================================================

/**
 * リリース前の事前チェックを実行する
 * @returns チェック結果の配列
 */
export async function checkPrerequisites(): Promise<CheckResult[]> {
  const checks: CheckResult[] = [];

  // ブランチチェック
  const branch = await getCurrentBranch();
  checks.push({
    passed: branch === "main",
    error:
      branch !== "main" ? `mainブランチにいません（現在: ${branch}）` : undefined,
  });

  // 未コミット変更チェック
  const hasChanges = await hasUncommittedChanges();
  checks.push({
    passed: !hasChanges,
    error: hasChanges ? "未コミットの変更があります" : undefined,
  });

  // リモート同期チェック
  const synced = await isSyncedWithRemote();
  checks.push({
    passed: synced,
    error: !synced ? "リモートと同期していません" : undefined,
  });

  return checks;
}
