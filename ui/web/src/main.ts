import { createApp } from "vue";
import { createPinia } from "pinia";
import router from "./router";
import i18n from "./i18n";
import App from "./App.vue";

// Import CSS files (order matters: Tabler first, then our custom styles)
import "@tabler/core/dist/css/tabler.min.css";
import "./styles/main.scss";

// Import and initialize theme system
import { initializeTheme, watchSystemTheme } from "./composables/useTheme";

// Initialize theme before creating the app
try {
  initializeTheme();
} catch (error) {
  console.error("Failed to initialize theme:", error);
}

// Watch for system theme changes
try {
  watchSystemTheme();
} catch (error) {
  console.error("Failed to initialize system theme watcher:", error);
}

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);
app.use(i18n);

// Initialize settings store and sync locale
import { useSettingsStore } from "./stores/settings";
import { storeToRefs } from "pinia";
const settingsStore = useSettingsStore();
settingsStore.initializeSettings();

// Sync i18n locale with settings store
const { locale } = storeToRefs(settingsStore);
i18n.global.locale.value = locale.value;

app.mount("#app");
