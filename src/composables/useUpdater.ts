import { ref } from 'vue';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export type UpdateState =
  | 'idle'
  | 'checking'
  | 'available'
  | 'downloading'
  | 'installing'
  | 'uptodate'
  | 'error';

export function useUpdater() {
  const state = ref<UpdateState>('idle');
  const newVersion = ref('');
  const errorMsg = ref('');
  const progress = ref(0);
  /** Release notes of the pending update (from latest.json's notes field). */
  const updateNotes = ref('');
  let pending: Update | null = null;

  async function checkForUpdates() {
    state.value = 'checking';
    errorMsg.value = '';
    progress.value = 0;
    updateNotes.value = '';
    try {
      const update = await check();
      pending = update;
      if (update) {
        state.value = 'available';
        newVersion.value = update.version;
        updateNotes.value = update.body ?? '';
        if (!updateNotes.value) {
          // Some endpoints omit notes; fall back to the GitHub release body.
          try {
            const resp = await fetch('https://api.github.com/repos/vswk001/Taxa/releases/latest');
            if (resp.ok) {
              const release = (await resp.json()) as { body?: string };
              updateNotes.value = release.body ?? '';
            }
          } catch {
            /* notes are optional — the version banner still works */
          }
        }
      } else {
        state.value = 'uptodate';
      }
    } catch (e: unknown) {
      state.value = 'error';
      errorMsg.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function downloadAndInstall() {
    if (!pending) return;
    state.value = 'downloading';
    progress.value = 0;
    let total = 0;
    let downloaded = 0;
    try {
      await pending.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            total = event.data.contentLength ?? 0;
            break;
          case 'Progress':
            downloaded += event.data.chunkLength ?? 0;
            progress.value = total ? Math.min(100, Math.round((downloaded / total) * 100)) : 0;
            break;
          case 'Finished':
            state.value = 'installing';
            break;
        }
      });
      // Install completed — restart into the new version.
      state.value = 'installing';
      await relaunch();
    } catch (e: unknown) {
      state.value = 'error';
      errorMsg.value = e instanceof Error ? e.message : String(e);
    }
  }

  return { state, newVersion, errorMsg, progress, updateNotes, checkForUpdates, downloadAndInstall };
}
