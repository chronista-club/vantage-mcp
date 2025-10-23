import { createApp } from 'vue';
import { createPinia } from 'pinia';
import router from './router';
import i18n from './i18n';
import App from './App.vue';

// Import CSS files
import '@tabler/core/dist/css/tabler.min.css';

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);
app.use(i18n);

// Initialize settings store and sync locale
import { useSettingsStore } from './stores/settings';
const settingsStore = useSettingsStore();
settingsStore.initializeSettings();

// Sync i18n locale with settings store
i18n.global.locale.value = settingsStore.locale;

app.mount('#app');