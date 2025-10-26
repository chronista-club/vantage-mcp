<template>
  <div class="settings-dropdown" ref="dropdownRef">
    <button
      @click="toggleDropdown"
      class="settings-btn"
      :aria-label="t('header.settings')"
      :aria-expanded="isOpen"
    >
      <IconSettings :size="18" :stroke-width="2" />
    </button>

    <Transition name="dropdown">
      <div v-if="isOpen" class="dropdown-menu">
        <div class="dropdown-section">
          <div class="dropdown-label">{{ t('settings.theme') }}</div>
          <button @click="toggleTheme" class="dropdown-item">
            <IconSun v-if="isDarkMode" :size="16" />
            <IconMoon v-else :size="16" />
            <span>{{ isDarkMode ? t('theme.light') : t('theme.dark') }}</span>
          </button>
        </div>

        <div class="dropdown-divider"></div>

        <div class="dropdown-section">
          <div class="dropdown-label">{{ t('settings.language') }}</div>
          <button @click="toggleLanguage" class="dropdown-item">
            <IconLanguage :size="16" />
            <span>{{ currentLocale === 'ja' ? 'English' : '日本語' }}</span>
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { IconSettings, IconSun, IconMoon, IconLanguage } from '@tabler/icons-vue';
import { useSettingsStore } from '@/stores/settings';
import { useI18n } from 'vue-i18n';
import { storeToRefs } from 'pinia';

const settingsStore = useSettingsStore();
const { locale: currentLocale } = storeToRefs(settingsStore);
const { t, locale } = useI18n();

const isOpen = ref(false);
const dropdownRef = ref<HTMLElement | null>(null);

const isDarkMode = computed(() => settingsStore.isDarkMode);

function toggleDropdown() {
  isOpen.value = !isOpen.value;
}

function toggleTheme() {
  settingsStore.toggleTheme();
  isOpen.value = false;
}

function toggleLanguage() {
  const newLocale = currentLocale.value === 'ja' ? 'en' : 'ja';
  locale.value = newLocale;
  settingsStore.setLocale(newLocale);
  isOpen.value = false;
}

function handleClickOutside(event: MouseEvent) {
  if (dropdownRef.value && !dropdownRef.value.contains(event.target as Node)) {
    isOpen.value = false;
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});
</script>

<style scoped lang="scss">
.settings-dropdown {
  position: relative;
}

.settings-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: oklch(0.5 0 0);
  cursor: pointer;
  transition: all 0.15s ease;

  @media (prefers-color-scheme: dark) {
    color: oklch(0.65 0 0);
  }

  &:hover {
    background: oklch(0.95 0 0);
    color: oklch(0.2 0 0);

    @media (prefers-color-scheme: dark) {
      background: oklch(0.25 0 0);
      color: oklch(0.9 0 0);
    }
  }

  &:active {
    transform: scale(0.95);
  }

  &:focus-visible {
    outline: 2px solid var(--vantage-btn-primary-bg);
    outline-offset: 2px;
  }
}

.dropdown-menu {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  min-width: 200px;
  background: oklch(1 0 0);
  border: 1px solid oklch(0.9 0 0);
  border-radius: 8px;
  box-shadow: 0 4px 12px oklch(0 0 0 / 0.1);
  padding: 8px;
  z-index: 1000;

  @media (prefers-color-scheme: dark) {
    background: oklch(0.22 0 0);
    border-color: oklch(0.3 0 0);
    box-shadow: 0 4px 12px oklch(0 0 0 / 0.4);
  }
}

.dropdown-section {
  padding: 4px 0;
}

.dropdown-label {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: oklch(0.55 0 0);
  padding: 4px 12px;
  margin-bottom: 4px;

  @media (prefers-color-scheme: dark) {
    color: oklch(0.6 0 0);
  }
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: oklch(0.3 0 0);
  font-size: 0.875rem;
  text-align: left;
  cursor: pointer;
  transition: all 0.15s ease;

  @media (prefers-color-scheme: dark) {
    color: oklch(0.85 0 0);
  }

  &:hover {
    background: oklch(0.96 0 0);

    @media (prefers-color-scheme: dark) {
      background: oklch(0.26 0 0);
    }
  }

  &:active {
    background: oklch(0.94 0 0);

    @media (prefers-color-scheme: dark) {
      background: oklch(0.24 0 0);
    }
  }

  span {
    flex: 1;
  }
}

.dropdown-divider {
  height: 1px;
  background: oklch(0.92 0 0);
  margin: 8px 0;

  @media (prefers-color-scheme: dark) {
    background: oklch(0.28 0 0);
  }
}

// Dropdown transition
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.2s ease;
}

.dropdown-enter-from {
  opacity: 0;
  transform: translateY(-8px) scale(0.95);
}

.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(0.98);
}
</style>
