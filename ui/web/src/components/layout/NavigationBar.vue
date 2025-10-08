<template>
  <div class="navigation-tabs bg-light border-bottom">
    <div class="container-xl">
      <ul class="nav nav-tabs border-0">
        <li
          v-for="item in navigationItems"
          :key="item.name"
          class="nav-item"
        >
          <router-link
            :to="{ name: item.route }"
            class="nav-link d-flex align-items-center"
            :class="{ active: route.name === item.route }"
          >
            <component :is="item.icon" class="me-2 nav-icon" />
            <span class="nav-label">{{ item.label }}</span>
          </router-link>
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router';
import {
  IconActivity,
  IconTemplate,
  IconClipboard
} from '@tabler/icons-vue';

const route = useRoute();

const navigationItems = [
  {
    name: 'processes',
    route: 'processes',
    label: 'Processes',
    icon: IconActivity,
  },
  {
    name: 'templates',
    route: 'templates',
    label: 'Templates',
    icon: IconTemplate,
  },
  {
    name: 'clipboard',
    route: 'clipboard',
    label: 'Clipboard',
    icon: IconClipboard,
  },
];
</script>

<style scoped>
.navigation-tabs {
  position: relative;
}

.nav-tabs {
  margin-bottom: 0;
}

.nav-link {
  color: var(--tblr-secondary);
  border: none;
  border-bottom: 2px solid transparent;
  padding: 0.75rem 1.25rem;
  transition: all 0.2s ease;
  font-weight: 500;
}

.nav-link:hover {
  color: var(--tblr-primary);
  background-color: rgba(var(--tblr-primary-rgb), 0.05);
  border-bottom-color: transparent;
}

.nav-link.active {
  color: var(--tblr-primary);
  background-color: white;
  border-bottom-color: var(--tblr-primary);
  box-shadow: 0 -1px 0 0 rgba(0, 0, 0, 0.05);
}

.nav-icon {
  width: 1.25rem;
  height: 1.25rem;
}

@media (max-width: 768px) {
  .nav-label {
    display: none;
  }

  .nav-icon {
    margin-right: 0 !important;
  }

  .nav-link {
    padding: 0.75rem;
  }
}
</style>