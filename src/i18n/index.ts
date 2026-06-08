import { createI18n } from 'vue-i18n';
import zhCN from './locales/zh-CN';
import zhTW from './locales/zh-TW';
import en from './locales/en';
import es from './locales/es';
import ar from './locales/ar';
import pt from './locales/pt';
import ja from './locales/ja';
import fr from './locales/fr';
import de from './locales/de';

const saved = localStorage.getItem('taxis-locale');
const browserLang = navigator.language;

let initialLocale = saved || 'zh-CN';
if (!saved) {
  const map: Record<string, string> = {
    'zh': 'zh-CN', 'zh-CN': 'zh-CN', 'zh-Hans': 'zh-CN', 'zh-Hans-CN': 'zh-CN',
    'zh-TW': 'zh-TW', 'zh-Hant': 'zh-TW', 'zh-Hant-TW': 'zh-TW',
    'en': 'en', 'en-US': 'en', 'en-GB': 'en',
    'es': 'es', 'es-ES': 'es',
    'ar': 'ar', 'ar-SA': 'ar',
    'pt': 'pt', 'pt-BR': 'pt', 'pt-PT': 'pt',
    'ja': 'ja', 'ja-JP': 'ja',
    'fr': 'fr', 'fr-FR': 'fr',
    'de': 'de', 'de-DE': 'de',
  };
  initialLocale = map[browserLang] || 'zh-CN';
}

const i18n = createI18n({
  legacy: false,
  locale: initialLocale,
  fallbackLocale: 'zh-CN',
  messages: {
    'zh-CN': zhCN,
    'zh-TW': zhTW,
    en,
    es,
    ar,
    pt,
    ja,
    fr,
    de,
  },
});

export const RTL_LANGUAGES = ['ar'];

export function setLocale(lang: string) {
  i18n.global.locale.value = lang as any;
  localStorage.setItem('taxis-locale', lang);
  if (RTL_LANGUAGES.includes(lang)) {
    document.documentElement.setAttribute('dir', 'rtl');
  } else {
    document.documentElement.removeAttribute('dir');
  }
}

export const SUPPORTED_LOCALES = [
  { value: 'zh-CN', label: '简体中文' },
  { value: 'zh-TW', label: '繁體中文' },
  { value: 'en', label: 'English' },
  { value: 'es', label: 'Español' },
  { value: 'ar', label: 'العربية' },
  { value: 'pt', label: 'Português' },
  { value: 'ja', label: '日本語' },
  { value: 'fr', label: 'Français' },
  { value: 'de', label: 'Deutsch' },
] as const;

export default i18n;
