import { getRuntimeSurface, type RuntimeSurface } from './runtime';

export interface WindowTitleInput {
  projectTitle: string;
  dirty: boolean;
  surface?: RuntimeSurface;
}

export function formatWindowTitle({
  projectTitle,
  dirty,
  surface = getRuntimeSurface(),
}: WindowTitleInput): string {
  const title = projectTitle.trim() || 'Untitled';
  const dirtyMark = dirty ? '*' : '';
  const suffix = surface === 'desktop' ? 'TreeMaker' : 'TreeMaker Web';
  return `${dirtyMark}${title} - ${suffix}`;
}

export async function applyWindowTitle(title: string, surface: RuntimeSurface = getRuntimeSurface()) {
  if (typeof document !== 'undefined') {
    document.title = title;
  }

  if (surface !== 'desktop') return;

  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().setTitle(title);
  } catch (error) {
    console.warn('Failed to update Tauri window title', error);
  }
}
