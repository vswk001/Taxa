import { ref, watchEffect } from 'vue';

export type Theme = 'light' | 'dark' | 'system';

const theme = ref<Theme>((localStorage.getItem('taxis-theme') as Theme) || 'system');

export function useTheme() {
  function applyTheme(t: Theme) {
    let isDark = t === 'dark';
    if (t === 'system') {
      isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    }
    document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light');
    localStorage.setItem('taxis-theme', t);
  }

  watchEffect(() => applyTheme(theme.value));
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (theme.value === 'system') applyTheme('system');
  });

  return { theme };
}
