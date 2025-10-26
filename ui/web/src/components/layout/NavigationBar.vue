<template>
  <nav class="vantage-nav">
    <div class="container-xl">
      <ul class="nav-list">
        <li
          v-for="item in navigationItems"
          :key="item.name"
          class="nav-item"
        >
          <router-link
            :to="{ name: item.route }"
            class="nav-link"
            :class="{ active: route.name === item.route }"
          >
            <component
              :is="item.icon"
              :size="20"
              :stroke-width="2"
              class="nav-icon"
            />
            <span class="nav-label">{{ item.label }}</span>
          </router-link>
        </li>
      </ul>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { computed } from 'vue';
import {
  IconActivity,
  IconTemplate,
  IconClipboard
} from '@tabler/icons-vue';

const route = useRoute();
const { t } = useI18n();

const navigationItems = computed(() => [
  {
    name: 'processes',
    route: 'processes',
    label: t('navigation.processes'),
    icon: IconActivity,
  },
  {
    name: 'templates',
    route: 'templates',
    label: t('navigation.templates'),
    icon: IconTemplate,
  },
  {
    name: 'clipboard',
    route: 'clipboard',
    label: t('navigation.clipboard'),
    icon: IconClipboard,
  },
]);
</script>

<style scoped lang="scss">
.vantage-nav {
  background: oklch(0.98 0 0);
  border-bottom: 1px solid oklch(0.92 0 0);

  @media (prefers-color-scheme: dark) {
    background: oklch(0.18 0 0);
    border-bottom-color: oklch(0.25 0 0);
  }
}

.nav-list {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  list-style: none;
  margin: 0;
  padding: 0;
}

.nav-item {
  margin: 0;
}

.nav-link {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.875rem 1.25rem;
  text-decoration: none;
  color: oklch(0.5 0 0);
  font-size: 0.9375rem;
  font-weight: 500;
  letter-spacing: -0.01em;
  border-bottom: 2px solid transparent;
  transition: all 0.15s ease;
  position: relative;

  @media (prefers-color-scheme: dark) {
    color: oklch(0.65 0 0);
  }

  &:hover:not(.active) {
    color: oklch(0.3 0 0);
    background: oklch(0.96 0 0);

    @media (prefers-color-scheme: dark) {
      color: oklch(0.85 0 0);
      background: oklch(0.22 0 0);
    }
  }

  &.active {
    color: var(--vantage-btn-primary-bg);
    border-bottom-color: var(--vantage-btn-primary-bg);

    .nav-icon {
      opacity: 1;
    }

    @media (prefers-color-scheme: dark) {
      color: var(--vantage-btn-primary-bg);
    }
  }
}

.nav-icon {
  opacity: 0.7;
  transition: opacity 0.15s ease;
}

.nav-label {
  line-height: 1;
}

// Responsive Design
@media (max-width: 768px) {
  .nav-link {
    padding: 0.875rem 1rem;
    gap: 0.375rem;
  }

  .nav-label {
    font-size: 0.875rem;
  }
}

@media (max-width: 576px) {
  .nav-list {
    gap: 0;
  }

  .nav-link {
    padding: 0.875rem 0.75rem;
    justify-content: center;
  }

  .nav-label {
    display: none;
  }

  .nav-icon {
    margin: 0;
  }
}
</style>