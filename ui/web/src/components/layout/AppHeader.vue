<template>
  <header class="vantage-header">
    <div class="container-xl">
      <div class="header-content">
        <!-- Brand -->
        <router-link to="/" class="brand">
          <div class="brand-logo">
            <svg viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path
                d="M16 4L4 10L16 16L28 10L16 4Z"
                fill="currentColor"
                opacity="0.9"
              />
              <path
                d="M4 16L16 22L28 16"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                opacity="0.7"
              />
              <path
                d="M4 22L16 28L28 22"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                opacity="0.5"
              />
            </svg>
          </div>
          <div class="brand-text">
            <span class="brand-name">{{ t('header.brandName') }}</span>
            <span class="brand-subtitle">{{ t('header.brandSubtitle') }}</span>
          </div>
        </router-link>

        <!-- Actions -->
        <nav class="header-actions">
          <button
            @click="toggleLanguage"
            class="header-action-btn header-action-btn-lang"
            :title="currentLanguage === 'ja' ? 'Switch to English' : '日本語に切り替え'"
            :aria-label="currentLanguage === 'ja' ? 'Switch to English' : '日本語に切り替え'"
          >
            <span class="lang-text">{{ currentLanguage === 'ja' ? 'EN' : 'JA' }}</span>
          </button>

          <button
            @click="toggleTheme"
            class="header-action-btn"
            :title="themeToggleTitle"
            :aria-label="themeToggleTitle"
          >
            <IconSun v-if="isDarkMode" :size="20" :stroke-width="2" />
            <IconMoon v-else :size="20" :stroke-width="2" />
          </button>

          <button
            @click="showSettings"
            class="header-action-btn"
            :title="t('header.settings')"
            :aria-label="t('header.settings')"
          >
            <IconSettings :size="20" :stroke-width="2" />
          </button>
        </nav>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { IconSun, IconMoon, IconSettings } from '@tabler/icons-vue';
import { useSettingsStore } from '@/stores/settings';
import { useI18n } from 'vue-i18n';
import { computed } from 'vue';

const settingsStore = useSettingsStore();
const { t, locale } = useI18n();

const isDarkMode = computed(() => settingsStore.isDarkMode);
const currentLanguage = computed(() => settingsStore.locale);

const themeToggleTitle = computed(() => {
  const mode = isDarkMode.value ? t('theme.light') : t('theme.dark');
  return t('header.themeSwitch', { mode });
});

function toggleTheme() {
  settingsStore.toggleTheme();
}

function toggleLanguage() {
  const newLocale = currentLanguage.value === 'ja' ? 'en' : 'ja';
  locale.value = newLocale;
  settingsStore.setLocale(newLocale);
}

function showSettings() {
  // TODO: Implement settings modal
  console.log('Show settings');
}
</script>

<style scoped lang="scss">
.vantage-header {
  position: sticky;
  top: 0;
  z-index: 1000;
  background: oklch(1 0 0 / 0.8);
  backdrop-filter: blur(12px) saturate(180%);
  border-bottom: 1px solid oklch(0.92 0 0);
  transition: all 0.2s ease;

  @media (prefers-color-scheme: dark) {
    background: oklch(0.2 0 0 / 0.85);
    border-bottom-color: oklch(0.3 0 0);
  }
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 64px;
  gap: 2rem;
}

// Brand Section
.brand {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  text-decoration: none;
  color: inherit;
  transition: opacity 0.2s ease;

  &:hover {
    opacity: 0.8;

    .brand-logo {
      transform: translateY(-2px);
    }
  }

  &:active {
    opacity: 0.6;
  }
}

.brand-logo {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  color: var(--vantage-btn-primary-bg);
  transition: transform 0.2s ease;

  svg {
    width: 100%;
    height: 100%;
  }
}

.brand-text {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.brand-name {
  font-size: 1.25rem;
  font-weight: 700;
  line-height: 1;
  letter-spacing: -0.02em;
  color: oklch(0.2 0 0);

  @media (prefers-color-scheme: dark) {
    color: oklch(0.95 0 0);
  }
}

.brand-subtitle {
  font-size: 0.75rem;
  font-weight: 500;
  line-height: 1;
  letter-spacing: 0.02em;
  color: oklch(0.5 0 0);
  text-transform: uppercase;

  @media (prefers-color-scheme: dark) {
    color: oklch(0.6 0 0);
  }
}

// Header Actions
.header-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.header-action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: oklch(0.4 0 0);
  cursor: pointer;
  transition: all 0.15s ease;

  @media (prefers-color-scheme: dark) {
    color: oklch(0.7 0 0);
  }

  &:hover {
    background: oklch(0.95 0 0);
    color: oklch(0.2 0 0);
    transform: translateY(-1px);

    @media (prefers-color-scheme: dark) {
      background: oklch(0.25 0 0);
      color: oklch(0.9 0 0);
    }
  }

  &:active {
    transform: translateY(0);
    background: oklch(0.92 0 0);

    @media (prefers-color-scheme: dark) {
      background: oklch(0.22 0 0);
    }
  }

  &:focus-visible {
    outline: 2px solid var(--vantage-btn-primary-bg);
    outline-offset: 2px;
  }
}

.header-action-btn-lang {
  .lang-text {
    font-size: 0.6875rem;
    font-weight: 700;
    letter-spacing: 0.05em;
    line-height: 1;
  }
}

// Responsive Design
@media (max-width: 768px) {
  .header-content {
    height: 56px;
  }

  .brand-logo {
    width: 32px;
    height: 32px;
  }

  .brand-name {
    font-size: 1.125rem;
  }

  .brand-subtitle {
    font-size: 0.6875rem;
  }

  .header-action-btn {
    width: 36px;
    height: 36px;
  }
}
</style>