import { invoke } from '@tauri-apps/api/core';

export interface QuickCaptureSettings {
  enabled: boolean;
  accelerator: string;
}

const STORAGE_KEY = 'taxa-quick-capture';
export const DEFAULT_ACCELERATOR = 'Alt+Shift+T';

export function loadQuickCaptureSettings(): QuickCaptureSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as Partial<QuickCaptureSettings>;
      return {
        enabled: parsed.enabled ?? true,
        accelerator: parsed.accelerator || DEFAULT_ACCELERATOR,
      };
    }
  } catch {
    /* fall through to defaults */
  }
  return { enabled: true, accelerator: DEFAULT_ACCELERATOR };
}

export function saveQuickCaptureSettings(settings: QuickCaptureSettings) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
}

/** Apply the stored settings to the backend (register/unregister). */
export function applyQuickCaptureShortcut(settings: QuickCaptureSettings): Promise<unknown> {
  return invoke('set_quick_capture_shortcut', {
    accelerator: settings.enabled ? settings.accelerator : '',
  });
}
