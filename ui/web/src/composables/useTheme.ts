/**
 * テーマ管理用のVue Composable
 */

import { ref, computed } from "vue";
import {
  type VantageTheme,
  type OklchColor,
  createLightTheme,
  createDarkTheme,
  applyTheme,
  getCurrentTheme,
  oklchToString,
  adjustLightness,
  adjustChroma,
  adjustHue,
  withAlpha,
  getHoverColor,
  getActiveColor,
  colorPresets,
} from "@/themes";

/**
 * グローバルなテーマ状態
 * 初期化はinitializeThemeで行われるため、ここではデフォルト値を設定
 */
const currentTheme = ref<VantageTheme>(createLightTheme());
const isDark = computed(() => !currentTheme.value.isLight);

/**
 * テーマの初期化（main.tsから呼び出される）
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

  // refを更新してからDOMに適用
  currentTheme.value = theme;
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
 * テーマ管理用のComposable
 */
export function useTheme() {
  /**
   * テーマを切り替える
   */
  function toggleTheme(): void {
    const newTheme = currentTheme.value.isLight
      ? createDarkTheme()
      : createLightTheme();
    setTheme(newTheme);
  }

  /**
   * テーマを設定する
   */
  function setTheme(theme: VantageTheme): void {
    currentTheme.value = theme;
    applyTheme(theme);

    // HTML要素のクラスとローカルストレージを更新
    const html = document.documentElement;
    const themeMode = theme.isLight ? "light" : "dark";

    if (theme.isLight) {
      html.classList.remove("dark");
      html.setAttribute("data-bs-theme", "light");
    } else {
      html.classList.add("dark");
      html.setAttribute("data-bs-theme", "dark");
    }

    localStorage.setItem("vantage-theme", themeMode);
  }

  /**
   * ライトモードに切り替え
   */
  function setLightMode(): void {
    setTheme(createLightTheme());
  }

  /**
   * ダークモードに切り替え
   */
  function setDarkMode(): void {
    setTheme(createDarkTheme());
  }

  /**
   * 色をCSS変数名として取得
   */
  function getColorVar(name: keyof VantageTheme["colors"]): string {
    return `var(--vantage-color-${name})`;
  }

  /**
   * 色を直接取得
   */
  function getColor(name: keyof VantageTheme["colors"]): OklchColor {
    return currentTheme.value.colors[name];
  }

  /**
   * 色をCSS文字列として取得
   */
  function getColorString(name: keyof VantageTheme["colors"]): string {
    return oklchToString(currentTheme.value.colors[name]);
  }

  return {
    // State
    currentTheme: computed(() => currentTheme.value),
    isDark,
    isLight: computed(() => currentTheme.value.isLight),

    // Actions
    toggleTheme,
    setTheme,
    setLightMode,
    setDarkMode,

    // Getters
    getColorVar,
    getColor,
    getColorString,

    // Utilities (re-export for convenience)
    oklchToString,
    adjustLightness,
    adjustChroma,
    adjustHue,
    withAlpha,
    getHoverColor,
    getActiveColor,
    colorPresets,
  };
}

/**
 * システムのカラースキーム変更を監視
 */
export function watchSystemTheme(callback?: (isDark: boolean) => void): void {
  if (!window.matchMedia) return;

  const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

  const handler = (e: MediaQueryListEvent) => {
    // ユーザーが明示的にテーマを設定している場合は無視
    if (localStorage.getItem("vantage-theme")) return;

    const newTheme = e.matches ? createDarkTheme() : createLightTheme();
    currentTheme.value = newTheme;
    applyTheme(newTheme);

    callback?.(e.matches);
  };

  mediaQuery.addEventListener("change", handler);
}
