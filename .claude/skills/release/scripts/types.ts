// types.ts - リリーススクリプトの型定義

/**
 * セマンティックバージョン
 * 例: 0.1.0-beta20 → { major: 0, minor: 1, patch: 0, prerelease: "beta20" }
 */
export type Version = {
  readonly major: number;
  readonly minor: number;
  readonly patch: number;
  readonly prerelease?: string;
};

/**
 * リリース設定
 */
export type ReleaseConfig = {
  readonly newVersion: Version;
  readonly currentVersion: Version;
  readonly tag: string;
};

/**
 * チェック結果
 */
export type CheckResult = {
  readonly passed: boolean;
  readonly error?: string;
};
