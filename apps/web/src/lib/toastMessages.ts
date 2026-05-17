export type ProjectMessageToastKind = 'info' | 'success' | 'warning' | 'error';

export interface ProjectMessageToast {
  kind: ProjectMessageToastKind;
  title: string;
  message?: string;
}

export function formatUnknownError(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  if (error && typeof error === 'object' && 'message' in error) {
    const message = (error as { message?: unknown }).message;
    if (typeof message === 'string') return message;
  }
  return String(error);
}

export function toastFromProjectMessage(message: string): ProjectMessageToast {
  if (message.startsWith('Saved ')) {
    return { kind: 'success', title: 'Project saved', message };
  }
  if (message.startsWith('Exported ')) {
    return { kind: 'success', title: 'Export complete', message };
  }
  if (message.startsWith('Loaded ')) {
    return { kind: 'success', title: 'Project loaded', message };
  }
  if (message.startsWith('Copied ')) {
    return { kind: 'info', title: 'Selection copied', message };
  }
  if (message.startsWith('Pasted ')) {
    return { kind: 'success', title: 'Selection pasted', message };
  }
  if (message.startsWith('Undid ')) {
    return { kind: 'info', title: 'Undo', message };
  }
  if (message.startsWith('Redid ')) {
    return { kind: 'info', title: 'Redo', message };
  }
  if (message.startsWith('Cleared ')) {
    return { kind: 'info', title: 'Cleared', message };
  }
  if (message.startsWith('Optimize ') || message.includes('optimization')) {
    return { kind: 'success', title: 'Optimization complete', message };
  }
  if (message.includes('crease pattern')) {
    return { kind: 'success', title: 'Crease pattern ready', message };
  }
  return { kind: 'info', title: message };
}
