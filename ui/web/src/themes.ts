/**
 * Vantage Design System - OKLCH色空間ベースのテーマシステム
 *
 * OKLCH色空間の利点:
 * - 知覚的に均一な色空間 (人間の目に自然な色の変化)
 * - 明度(L)、彩度(C)、色相(H)を独立して操作可能
 * - 広い色域をカバー
 * - ダークモード対応が容易
 */

/**
 * OKLCH色の型定義
 */
export interface OklchColor {
  /** 明度 (Lightness): 0-1 の範囲 */
  l: number;
  /** 彩度 (Chroma): 0-0.4 の範囲 (0.4以上も可能だが、一般的には0.4まで) */
  c: number;
  /** 色相 (Hue): 0-360 の範囲 (度数) */
  h: number;
  /** 不透明度 (Alpha): 0-1 の範囲 */
  a?: number;
}

/**
 * テーマカラーパレット
 */
export interface ThemeColorPalette {
  /** プライマリカラー (メインブランドカラー) */
  primary: OklchColor;
  /** セカンダリカラー (サポートカラー) */
  secondary: OklchColor;
  /** アクセントカラー (強調) */
  accent: OklchColor;
  /** 成功カラー */
  success: OklchColor;
  /** 警告カラー */
  warning: OklchColor;
  /** エラーカラー */
  error: OklchColor;
  /** 情報カラー */
  info: OklchColor;
  /** グレースケール */
  gray: OklchColor;
  /** 背景色 */
  background: OklchColor;
  /** 前景色（テキストなど） */
  foreground: OklchColor;
  /** ボーダーカラー */
  border: OklchColor;
}

/**
 * テーマ設定
 */
export interface VantageTheme {
  /** テーマ名 */
  name: string;
  /** ライトモードか */
  isLight: boolean;
  /** カラーパレット */
  colors: ThemeColorPalette;
}

/**
 * OKLCH色をCSS文字列に変換
 */
export function oklchToString(color: OklchColor): string {
  const { l, c, h, a = 1 } = color;
  if (a < 1) {
    return `oklch(${l.toFixed(4)} ${c.toFixed(4)} ${h.toFixed(2)} / ${a.toFixed(2)})`;
  }
  return `oklch(${l.toFixed(4)} ${c.toFixed(4)} ${h.toFixed(2)})`;
}

/**
 * CSS文字列からOKLCH色をパース
 */
export function parseOklch(str: string): OklchColor | null {
  // oklch(0.65 0.12 240) or oklch(0.65 0.12 240 / 0.5)
  const match = str.match(
    /oklch\(([\d.]+)\s+([\d.]+)\s+([\d.]+)(?:\s*\/\s*([\d.]+))?\)/,
  );
  if (!match) return null;

  return {
    l: parseFloat(match[1]!),
    c: parseFloat(match[2]!),
    h: parseFloat(match[3]!),
    a: match[4] ? parseFloat(match[4]) : 1,
  };
}

/**
 * 明度を調整（ダークモード対応など）
 */
export function adjustLightness(color: OklchColor, delta: number): OklchColor {
  return {
    ...color,
    l: Math.max(0, Math.min(1, color.l + delta)),
  };
}

/**
 * 彩度を調整
 */
export function adjustChroma(color: OklchColor, delta: number): OklchColor {
  return {
    ...color,
    c: Math.max(0, Math.min(0.4, color.c + delta)),
  };
}

/**
 * 色相を調整
 */
export function adjustHue(color: OklchColor, delta: number): OklchColor {
  let newHue = color.h + delta;
  // 0-360の範囲に正規化
  while (newHue < 0) newHue += 360;
  while (newHue >= 360) newHue -= 360;

  return {
    ...color,
    h: newHue,
  };
}

/**
 * アルファ値を設定
 */
export function withAlpha(color: OklchColor, alpha: number): OklchColor {
  return {
    ...color,
    a: Math.max(0, Math.min(1, alpha)),
  };
}

/**
 * ホバー状態の色を生成（明度を少し下げる）
 */
export function getHoverColor(
  color: OklchColor,
  isLight: boolean = true,
): OklchColor {
  return adjustLightness(color, isLight ? -0.05 : 0.05);
}

/**
 * アクティブ状態の色を生成（明度をさらに下げる）
 */
export function getActiveColor(
  color: OklchColor,
  isLight: boolean = true,
): OklchColor {
  return adjustLightness(color, isLight ? -0.1 : 0.1);
}

/**
 * ダークモード用の色を生成
 */
export function toDarkMode(color: OklchColor): OklchColor {
  // ライトモードの色をダークモードに変換
  // 一般的に明度を下げ、彩度を少し下げる
  return {
    ...color,
    l: Math.max(0.2, color.l - 0.1),
    c: Math.max(0, color.c - 0.02),
  };
}

/**
 * ライトモード用の色を生成
 */
export function toLightMode(color: OklchColor): OklchColor {
  // ダークモードの色をライトモードに変換
  return {
    ...color,
    l: Math.min(0.9, color.l + 0.1),
    c: Math.min(0.4, color.c + 0.02),
  };
}

/**
 * グラデーションを生成（指定した色の明度を段階的に変化）
 */
export function generateLightnessScale(
  color: OklchColor,
  steps: number = 10,
): OklchColor[] {
  const scale: OklchColor[] = [];
  for (let i = 0; i < steps; i++) {
    const lightness = i / (steps - 1); // 0 から 1
    scale.push({
      ...color,
      l: lightness,
    });
  }
  return scale;
}

/**
 * 補色を生成
 */
export function getComplementary(color: OklchColor): OklchColor {
  return adjustHue(color, 180);
}

/**
 * 類似色を生成（隣接色相）
 */
export function getAnalogous(color: OklchColor): [OklchColor, OklchColor] {
  return [adjustHue(color, 30), adjustHue(color, -30)];
}

/**
 * 三色配色を生成
 */
export function getTriadic(color: OklchColor): [OklchColor, OklchColor] {
  return [adjustHue(color, 120), adjustHue(color, 240)];
}

/**
 * Vantageのライトテーマを生成
 * Contrastデザインシステムベース
 */
export function createLightTheme(): VantageTheme {
  return {
    name: "vantage-light",
    isLight: true,
    colors: {
      // プライマリ: Develop (青紫系) - #465BB9相当
      primary: { l: 0.48, c: 0.13, h: 280 },
      // セカンダリ: Research (ディープブルー) - #12126D相当
      secondary: { l: 0.28, c: 0.15, h: 285 },
      // アクセント: Broadcast (ピンク) - #EF6CDB相当
      accent: { l: 0.7, c: 0.2, h: 320 },
      // 成功: Provision (シアン) - #7DECF2相当
      success: { l: 0.85, c: 0.1, h: 195 },
      // 警告: 明るいゴールド
      warning: { l: 0.75, c: 0.15, h: 85 },
      // エラー: Creative (マゼンタ) - #F609AF相当
      error: { l: 0.65, c: 0.25, h: 330 },
      // 情報: ライトブルー
      info: { l: 0.65, c: 0.12, h: 240 },
      // グレー
      gray: { l: 0.6, c: 0.015, h: 280 },
      // 背景: ほぼ白
      background: { l: 0.98, c: 0.005, h: 280 },
      // 前景: ダークグレー
      foreground: { l: 0.2, c: 0.01, h: 280 },
      // ボーダー: ライトグレー
      border: { l: 0.88, c: 0.01, h: 280 },
    },
  };
}

/**
 * Vantageのダークテーマを生成
 * Contrastデザインシステムベース（ダークモード最適化）
 */
export function createDarkTheme(): VantageTheme {
  return {
    name: "vantage-dark",
    isLight: false,
    colors: {
      // プライマリ: Develop (青紫系・明度調整)
      primary: { l: 0.58, c: 0.13, h: 280 },
      // セカンダリ: Research (ディープブルー・明度調整)
      secondary: { l: 0.38, c: 0.15, h: 285 },
      // アクセント: Broadcast (ピンク・明度調整)
      accent: { l: 0.75, c: 0.2, h: 320 },
      // 成功: Provision (シアン・彩度調整)
      success: { l: 0.8, c: 0.12, h: 195 },
      // 警告: ゴールド（明度調整）
      warning: { l: 0.7, c: 0.15, h: 85 },
      // エラー: Creative (マゼンタ・明度調整)
      error: { l: 0.7, c: 0.23, h: 330 },
      // 情報: ブルー（明度調整）
      info: { l: 0.7, c: 0.12, h: 240 },
      // グレー
      gray: { l: 0.65, c: 0.015, h: 280 },
      // 背景: ダークグレー
      background: { l: 0.15, c: 0.01, h: 280 },
      // 前景: ライトグレー
      foreground: { l: 0.92, c: 0.01, h: 280 },
      // ボーダー: ミディアムグレー
      border: { l: 0.3, c: 0.01, h: 280 },
    },
  };
}

/**
 * テーマをCSS変数として適用
 */
export function applyTheme(theme: VantageTheme): void {
  const root = document.documentElement;

  // カラーパレットを適用
  Object.entries(theme.colors).forEach(([name, color]) => {
    root.style.setProperty(`--vantage-color-${name}`, oklchToString(color));

    // ホバー・アクティブ状態の色も生成
    root.style.setProperty(
      `--vantage-color-${name}-hover`,
      oklchToString(getHoverColor(color, theme.isLight)),
    );
    root.style.setProperty(
      `--vantage-color-${name}-active`,
      oklchToString(getActiveColor(color, theme.isLight)),
    );

    // アルファ変種も生成（オーバーレイなどに使用）
    root.style.setProperty(
      `--vantage-color-${name}-alpha-10`,
      oklchToString(withAlpha(color, 0.1)),
    );
    root.style.setProperty(
      `--vantage-color-${name}-alpha-20`,
      oklchToString(withAlpha(color, 0.2)),
    );
    root.style.setProperty(
      `--vantage-color-${name}-alpha-50`,
      oklchToString(withAlpha(color, 0.5)),
    );
  });

  // テーマモードを示すカスタムプロパティ
  root.style.setProperty(
    "--vantage-theme-mode",
    theme.isLight ? "light" : "dark",
  );
}

/**
 * 現在のテーマを取得
 */
export function getCurrentTheme(): VantageTheme {
  const isDark = document.documentElement.classList.contains("dark");
  return isDark ? createDarkTheme() : createLightTheme();
}

/**
 * テーマを切り替え
 */
export function toggleTheme(): void {
  const currentTheme = getCurrentTheme();
  const newTheme = currentTheme.isLight
    ? createDarkTheme()
    : createLightTheme();
  applyTheme(newTheme);

  // HTML要素のクラスも更新
  const html = document.documentElement;
  if (newTheme.isLight) {
    html.classList.remove("dark");
    html.setAttribute("data-bs-theme", "light");
  } else {
    html.classList.add("dark");
    html.setAttribute("data-bs-theme", "dark");
  }
}

/**
 * 初期化：システムのテーマ設定またはローカルストレージから読み込み
 */
export function initializeTheme(): void {
  const savedTheme = localStorage.getItem("vantage-theme") as
    | "light"
    | "dark"
    | null;
  let theme: VantageTheme;

  if (savedTheme) {
    theme = savedTheme === "dark" ? createDarkTheme() : createLightTheme();
  } else {
    // システムのカラースキームを確認
    const prefersDark = window.matchMedia(
      "(prefers-color-scheme: dark)",
    ).matches;
    theme = prefersDark ? createDarkTheme() : createLightTheme();
  }

  applyTheme(theme);

  // HTML要素のクラスを設定
  const html = document.documentElement;
  if (theme.isLight) {
    html.classList.remove("dark");
    html.setAttribute("data-bs-theme", "light");
  } else {
    html.classList.add("dark");
    html.setAttribute("data-bs-theme", "dark");
  }
}

/**
 * カスタムカラーを生成するヘルパー
 * Contrastデザインシステムベース
 */
export const colorPresets = {
  /** Develop: メイン開発機能 (青紫系) - #465BB9相当 */
  develop: { l: 0.48, c: 0.13, h: 280 },
  /** Research: セカンダリ・調査 (ディープブルー系) - #12126D相当 */
  research: { l: 0.28, c: 0.15, h: 285 },
  /** Broadcast: 削除・終了 (ピンク系) - #EF6CDB相当 */
  bradcast: { l: 0.7, c: 0.2, h: 320 },
  /** Creative: 実行・開始 (マゼンタ系) - #F609AF相当 */
  creative: { l: 0.65, c: 0.25, h: 330 },
  /** Provision: 配備・停止 (シアン系) - #7DECF2相当 */
  provision: { l: 0.85, c: 0.1, h: 195 },
  /** Finished: 完了・終了 (グレー系) */
  finished: { l: 0.6, c: 0.015, h: 280 },
} as const;

/**
 * デフォルトエクスポート
 */
export default {
  createLightTheme,
  createDarkTheme,
  applyTheme,
  getCurrentTheme,
  toggleTheme,
  initializeTheme,
  oklchToString,
  parseOklch,
  adjustLightness,
  adjustChroma,
  adjustHue,
  withAlpha,
  getHoverColor,
  getActiveColor,
  toDarkMode,
  toLightMode,
  generateLightnessScale,
  getComplementary,
  getAnalogous,
  getTriadic,
  colorPresets,
};
