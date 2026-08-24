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

const STORAGE_KEY = 'taxa-locale';
// One-time migration from the legacy pre-rename key.
if (!localStorage.getItem(STORAGE_KEY) && localStorage.getItem('taxis-locale')) {
  localStorage.setItem(STORAGE_KEY, localStorage.getItem('taxis-locale') as string);
}

const saved = localStorage.getItem(STORAGE_KEY);
const browserLang = navigator.language;

let initialLocale = saved || 'en';
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
  // Unmapped locales fall back to English, not Chinese.
  initialLocale = map[browserLang] || 'en';
}

const i18n = createI18n({
  legacy: false,
  locale: initialLocale,
  fallbackLocale: 'en',
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

/** Set document direction + lang so RTL layouts survive restarts (previously
 *  only applied when the user switched languages at runtime). */
function applyDocumentLocale(lang: string) {
  document.documentElement.lang = lang;
  if (RTL_LANGUAGES.includes(lang)) {
    document.documentElement.setAttribute('dir', 'rtl');
  } else {
    document.documentElement.setAttribute('dir', 'ltr');
  }
}

applyDocumentLocale(initialLocale);

export function setLocale(lang: string) {
  i18n.global.locale.value = lang as any;
  localStorage.setItem(STORAGE_KEY, lang);
  applyDocumentLocale(lang);
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
