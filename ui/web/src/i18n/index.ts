import { createI18n } from 'vue-i18n';
import en from './locales/en.json';
import ja from './locales/ja.json';

// デフォルトロケールの検出
const getDefaultLocale = (): string => {
  // ローカルストレージに保存された設定を確認
  const savedLocale = localStorage.getItem('vantage-locale');
  if (savedLocale && ['en', 'ja'].includes(savedLocale)) {
    return savedLocale;
  }

  // ブラウザの言語設定を確認
  const browserLocale = navigator.language.split('-')[0];
  if (browserLocale === 'en') {
    return 'en';
  }

  return 'ja'; // デフォルトは日本語
};

const i18n = createI18n({
  legacy: false, // Composition API を使用
  locale: getDefaultLocale(),
  fallbackLocale: 'en',
  messages: {
    en,
    ja,
  },
});

export default i18n;
