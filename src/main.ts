import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import { useTheme } from './composables/useTheme';
import './styles/global.css';

const app = createApp(App);
app.use(createPinia());
useTheme();
app.mount('#app');
