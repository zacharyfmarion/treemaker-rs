import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { isDesktopRuntime } from '../platform/runtime';

type OpenPathHandler = (path: string) => Promise<void>;

export function useTauriOpenedFiles(enabled: boolean, openPath: OpenPathHandler): void {
  useEffect(() => {
    if (!enabled || !isDesktopRuntime()) return;

    let disposed = false;
    let unlisten: (() => void) | null = null;

    const openFirstPath = (paths: string[]) => {
      const path = paths.find((candidate) => /\.osf$/i.test(candidate));
      if (!path || disposed) return;
      void openPath(path);
    };

    Promise.resolve()
      .then(async () => {
        const dispose = await listen<string[]>('opened-files', (event) => {
          openFirstPath(event.payload);
          void invoke<string[]>('take_opened_files').catch(() => undefined);
        });
        const initialPaths = await invoke<string[]>('take_opened_files');
        openFirstPath(initialPaths);
        return dispose;
      })
      .then((dispose) => {
        if (disposed) {
          dispose();
          return;
        }
        unlisten = dispose;
      })
      .catch((error) => {
        console.warn('Failed to register Tauri opened-file listener', error);
      });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [enabled, openPath]);
}
