<template>
  <div class="settings-view">
    <div class="container-xl">
      <!-- ページヘッダー -->
      <div class="page-header">
        <div class="header-content">
          <h1 class="page-title">{{ t('settings.title') }}</h1>
          <p class="text-muted">{{ t('settings.description') }}</p>
        </div>
      </div>

      <div class="row g-4">
        <!-- テーマ設定 -->
        <div class="col-12 col-lg-6">
          <div class="card">
            <div class="card-header">
              <h3 class="card-title">
                <IconPalette :size="20" :stroke-width="2" class="me-2" />
                {{ t('settings.theme.title') }}
              </h3>
            </div>
            <div class="card-body">
              <div class="mb-3">
                <label class="form-label">{{ t('settings.theme.mode') }}</label>
                <div class="btn-group w-100" role="group">
                  <input
                    type="radio"
                    class="btn-check"
                    name="theme-mode"
                    id="theme-light"
                    :checked="!isDark"
                    @change="setLightMode"
                  />
                  <label class="btn" for="theme-light">
                    <IconSun :size="18" :stroke-width="2" class="me-1" />
                    {{ t('settings.theme.light') }}
                  </label>

                  <input
                    type="radio"
                    class="btn-check"
                    name="theme-mode"
                    id="theme-dark"
                    :checked="isDark"
                    @change="setDarkMode"
                  />
                  <label class="btn" for="theme-dark">
                    <IconMoon :size="18" :stroke-width="2" class="me-1" />
                    {{ t('settings.theme.dark') }}
                  </label>
                </div>
              </div>

              <!-- カラープレビュー -->
              <div class="theme-preview">
                <h4 class="mb-3">{{ t('settings.theme.colorPalette') }}</h4>
                <div class="color-grid">
                  <div
                    v-for="colorName in colorNames"
                    :key="colorName"
                    class="color-item"
                  >
                    <div
                      class="color-swatch"
                      :style="{ backgroundColor: getColorVar(colorName as any) }"
                    ></div>
                    <span class="color-name">{{ colorName }}</span>
                    <code class="color-value">{{ getColorString(colorName as any) }}</code>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 表示設定 -->
        <div class="col-12 col-lg-6">
          <div class="card">
            <div class="card-header">
              <h3 class="card-title">
                <IconSettings :size="20" :stroke-width="2" class="me-2" />
                {{ t('settings.display.title') }}
              </h3>
            </div>
            <div class="card-body">
              <div class="mb-3">
                <label class="form-label">{{ t('settings.display.language') }}</label>
                <select
                  class="form-select"
                  :value="settingsStore.locale"
                  @change="onLocaleChange"
                >
                  <option value="ja">日本語</option>
                  <option value="en">English</option>
                </select>
              </div>

              <div class="mb-3">
                <label class="form-label">{{ t('settings.display.viewMode') }}</label>
                <div class="btn-group w-100" role="group">
                  <input
                    type="radio"
                    class="btn-check"
                    name="view-mode"
                    id="view-card"
                    :checked="settingsStore.viewMode === 'card'"
                    @change="() => settingsStore.setViewMode('card')"
                  />
                  <label class="btn" for="view-card">
                    <IconLayoutGrid :size="18" :stroke-width="2" class="me-1" />
                    {{ t('settings.display.cardView') }}
                  </label>

                  <input
                    type="radio"
                    class="btn-check"
                    name="view-mode"
                    id="view-table"
                    :checked="settingsStore.viewMode === 'table'"
                    @change="() => settingsStore.setViewMode('table')"
                  />
                  <label class="btn" for="view-table">
                    <IconTable :size="18" :stroke-width="2" class="me-1" />
                    {{ t('settings.display.tableView') }}
                  </label>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- OKLCH色操作デモ -->
        <div class="col-12">
          <div class="card">
            <div class="card-header">
              <h3 class="card-title">
                <IconAdjustments :size="20" :stroke-width="2" class="me-2" />
                {{ t('settings.colorDemo.title') }}
              </h3>
            </div>
            <div class="card-body">
              <p class="text-muted mb-3">{{ t('settings.colorDemo.description') }}</p>

              <div class="row g-3">
                <div class="col-12 col-md-4">
                  <label class="form-label">{{ t('settings.colorDemo.lightness') }}</label>
                  <input
                    type="range"
                    class="form-range"
                    min="0"
                    max="100"
                    v-model="demoLightness"
                  />
                  <small class="text-muted">{{ (demoLightness / 100).toFixed(2) }}</small>
                </div>

                <div class="col-12 col-md-4">
                  <label class="form-label">{{ t('settings.colorDemo.chroma') }}</label>
                  <input
                    type="range"
                    class="form-range"
                    min="0"
                    max="40"
                    v-model="demoChroma"
                  />
                  <small class="text-muted">{{ (demoChroma / 100).toFixed(2) }}</small>
                </div>

                <div class="col-12 col-md-4">
                  <label class="form-label">{{ t('settings.colorDemo.hue') }}</label>
                  <input
                    type="range"
                    class="form-range"
                    min="0"
                    max="360"
                    v-model="demoHue"
                  />
                  <small class="text-muted">{{ demoHue }}°</small>
                </div>
              </div>

              <div class="demo-color-preview mt-4">
                <div
                  class="demo-swatch"
                  :style="{ backgroundColor: demoColorString }"
                >
                  <code class="demo-code">{{ demoColorString }}</code>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  IconPalette,
  IconSun,
  IconMoon,
  IconSettings,
  IconLayoutGrid,
  IconTable,
  IconAdjustments,
} from '@tabler/icons-vue';
import { useTheme } from '@/composables/useTheme';
import { useSettingsStore } from '@/stores/settings';

const { t, locale } = useI18n();
const settingsStore = useSettingsStore();

const {
  isDark,
  setLightMode,
  setDarkMode,
  getColorVar,
  getColorString,
  oklchToString,
} = useTheme();

// カラーパレット一覧
const colorNames = [
  'primary',
  'secondary',
  'accent',
  'success',
  'warning',
  'error',
  'info',
  'gray',
];

// ロケール変更
function onLocaleChange(event: Event) {
  const target = event.target as HTMLSelectElement;
  const newLocale = target.value as 'ja' | 'en';
  settingsStore.setLocale(newLocale);
  locale.value = newLocale;
}

// OKLCH色デモ用の状態
const demoLightness = ref(65);
const demoChroma = ref(12);
const demoHue = ref(308);

const demoColorString = computed(() => {
  return oklchToString({
    l: demoLightness.value / 100,
    c: demoChroma.value / 100,
    h: demoHue.value,
  });
});
</script>

<style scoped lang="scss">
.settings-view {
  padding: 2rem 0;
}

.page-header {
  margin-bottom: 2rem;

  .header-content {
    .page-title {
      font-size: 2rem;
      font-weight: 600;
      margin-bottom: 0.5rem;
    }
  }
}

.theme-preview {
  margin-top: 1.5rem;

  h4 {
    font-size: 0.875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--tblr-muted);
  }
}

.color-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 1rem;
}

.color-item {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0.75rem;
  border: 1px solid var(--vantage-color-border);
  border-radius: 0.5rem;
  background: var(--tblr-card-bg);

  .color-swatch {
    width: 100%;
    height: 4rem;
    border-radius: 0.375rem;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .color-name {
    font-size: 0.875rem;
    font-weight: 500;
    text-transform: capitalize;
  }

  .color-value {
    font-size: 0.75rem;
    color: var(--tblr-muted);
    word-break: break-all;
  }
}

.demo-color-preview {
  .demo-swatch {
    width: 100%;
    height: 8rem;
    border-radius: 0.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);

    .demo-code {
      padding: 0.5rem 1rem;
      background: rgba(0, 0, 0, 0.5);
      color: white;
      border-radius: 0.375rem;
      font-size: 0.875rem;
      backdrop-filter: blur(8px);
    }
  }
}

// ダークモード対応
.dark {
  .color-item {
    background: var(--tblr-card-bg-dark, #1a1d21);
  }
}
</style>
