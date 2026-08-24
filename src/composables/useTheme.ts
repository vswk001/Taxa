import { ref, watchEffect } from 'vue';

export type Theme = 'light' | 'dark' | 'system';

const STORAGE_KEY = 'taxa-theme';
// One-time migration from the legacy pre-rename key.
if (!localStorage.getItem(STORAGE_KEY) && localStorage.getItem('taxis-theme')) {
  localStorage.setItem(STORAGE_KEY, localStorage.getItem('taxis-theme') as string);
}

const theme = ref<Theme>((localStorage.getItem(STORAGE_KEY) as Theme) || 'system');

function applyTheme(t: Theme) {
  let isDark = t === 'dark';
  if (t === 'system') {
    isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  }
  document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light');
  localStorage.setItem(STORAGE_KEY, t);
}

const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
function onMediaChange() {
  if (theme.value === 'system') applyTheme('system');
}
mediaQuery.addEventListener('change', onMediaChange);

/** Change the theme from anywhere (settings dialog, etc.) — persists and
 *  reacts through the shared ref instead of touching the DOM ad hoc. */
function setTheme(t: Theme) {
  theme.value = t;
}

export function useTheme() {
  watchEffect(() => applyTheme(theme.value));
  return { theme, setTheme };
}
