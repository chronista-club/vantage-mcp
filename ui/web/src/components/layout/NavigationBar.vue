<template>
  <div class="navbar-expand-md">
    <div class="navbar navbar-light">
      <div class="container-xl">
        <ul class="navbar-nav">
          <li 
            v-for="item in navigationItems"
            :key="item.name"
            class="nav-item"
            :class="{ active: route.name === item.route }"
          >
            <router-link 
              :to="{ name: item.route }"
              class="nav-link"
            >
              <span class="nav-link-icon d-md-none d-lg-inline-block">
                <component :is="item.icon" />
              </span>
              <span class="nav-link-title">{{ item.label }}</span>
            </router-link>
          </li>
        </ul>
      </div>
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