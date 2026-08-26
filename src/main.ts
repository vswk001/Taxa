import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import QuickCapture from './components/capture/QuickCapture.vue';
import { useTheme } from './composables/useTheme';
import i18n from './i18n';
import './styles/global.css';

// The quick-capture window loads the same bundle with #quick-capture; it
// mounts a tiny standalone root instead of the full app layout.
const isQuickCapture = window.location.hash === '#quick-capture';

const app = createApp(isQuickCapture ? QuickCapture : App);
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
