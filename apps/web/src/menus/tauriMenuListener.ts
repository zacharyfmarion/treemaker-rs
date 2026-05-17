import { useEffect } from 'react';
import { handleMenuAction } from '../commands/menuActions';
import { isDesktopRuntime } from '../platform/runtime';

export function useTauriMenuListener(): void {
  useEffect(() => {
    if (!isDesktopRuntime()) return;

    let disposed = false;
    let unlisten: (() => void) | null = null;

    import('@tauri-apps/api/event')
      .then(({ listen }) =>
        listen<string>('menu-action', (event) => {
          void handleMenuAction(event.payload);
        })
      )
      .then((dispose) => {
        if (disposed) {
          dispose();
          return;
        }
        unlisten = dispose;
      })
      .catch((error) => {
        console.warn('Failed to register Tauri menu listener', error);
      });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
}
