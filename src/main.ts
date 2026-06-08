import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import { useTheme } from './composables/useTheme';
import i18n from './i18n';
import './styles/global.css';

const app = createApp(App);
app.use(createPinia());
app.use(i18n);
useTheme();
app.mount('#app');
