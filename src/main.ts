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

// Suppress the browser/webview context menu in production builds. Done in the
// capture phase so it wins even if a handler downstream stops propagation.
// The app's own context menus (e.g. the note tree) are separate components
// that render their own UI, so they keep working. In dev we leave the native
// menu so right-click → Inspect still works.
if (import.meta.env.PROD) {
  document.addEventListener('contextmenu', (e) => e.preventDefault(), true);
}
