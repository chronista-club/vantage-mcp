import { createRouter, createWebHistory } from 'vue-router';
import type { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    redirect: '/processes',
  },
  {
    path: '/processes',
    name: 'processes',
    component: () => import('@/views/ProcessesView.vue'),
  },
  {
    path: '/templates',
    name: 'templates',
    component: () => import('@/views/TemplatesView.vue'),
  },
  {
    path: '/clipboard',
    name: 'clipboard',
    component: () => import('@/views/ClipboardView.vue'),
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'not-found',
    redirect: '/processes',
  },
];

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
});

export default router;