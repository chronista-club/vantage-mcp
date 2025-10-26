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
import { storeToRefs } from 'pinia';
const settingsStore = useSettingsStore();
settingsStore.initializeSettings();

// Sync i18n locale with settings store
const { locale } = storeToRefs(settingsStore);
i18n.global.locale.value = locale.value;

app.mount('#app');