import { createRouter, createWebHistory } from "vue-router";
import type { RouteRecordRaw } from "vue-router";

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    redirect: "/dashboard",
  },
  {
    path: "/dashboard",
    name: "dashboard",
    component: () => import("@/views/DashboardView.vue"),
  },
  {
    path: "/processes",
    name: "processes",
    component: () => import("@/views/ProcessesView.vue"),
  },
  {
    path: "/templates",
    name: "templates",
    component: () => import("@/views/TemplatesView.vue"),
  },
  {
    path: "/clipboard",
    name: "clipboard",
    component: () => import("@/views/ClipboardView.vue"),
  },
  {
    path: "/settings",
    name: "settings",
    component: () => import("@/views/SettingsView.vue"),
  },
  {
    path: "/:pathMatch(.*)*",
    name: "not-found",
    redirect: "/dashboard",
  },
];

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
});

export default router;
