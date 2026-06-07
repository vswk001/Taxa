import { ref, watchEffect } from 'vue';

export type Theme = 'light' | 'dark' | 'system';

const theme = ref<Theme>((localStorage.getItem('taxis-theme') as Theme) || 'system');

function applyTheme(t: Theme) {
  let isDark = t === 'dark';
  if (t === 'system') {
    isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  }
  document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light');
  localStorage.setItem('taxis-theme', t);
}

const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
function onMediaChange() {
  if (theme.value === 'system') applyTheme('system');
}
mediaQuery.addEventListener('change', onMediaChange);

export function useTheme() {
  watchEffect(() => applyTheme(theme.value));
  return { theme };
}
