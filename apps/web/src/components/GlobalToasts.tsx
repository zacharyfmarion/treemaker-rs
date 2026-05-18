import { useEffect, useRef } from 'react';
import { toast } from 'sonner';
import { formatUnknownError, toastFromProjectMessage } from '../lib/toastMessages';
import { useWorkspaceStore } from '../store/workspaceStore';

function errorKey(error: unknown): string {
  if (error && typeof error === 'object' && 'code' in error && 'message' in error) {
    const envelope = error as { code: unknown; message: unknown };
    return `${String(envelope.code)}:${String(envelope.message)}`;
  }
  return formatUnknownError(error);
}

export function GlobalToasts() {
  const error = useWorkspaceStore((state) => state.error);
  const projectMessage = useWorkspaceStore((state) => state.projectMessage);
  const clearProjectMessage = useWorkspaceStore((state) => state.clearProjectMessage);
  const lastErrorKey = useRef<string | null>(null);

  useEffect(() => {
    if (!error) {
      lastErrorKey.current = null;
      return;
    }

    const key = errorKey(error);
    if (lastErrorKey.current === key) return;
    lastErrorKey.current = key;

    toast.error('TreeMaker error', {
      id: `treemaker-error-${key}`,
      description: formatUnknownError(error),
      duration: 8000,
    });
  }, [error]);

  useEffect(() => {
    if (!projectMessage) return;

    const next = toastFromProjectMessage(projectMessage);
    toast[next.kind](next.title, {
      description: next.message,
      duration: next.kind === 'error' ? 8000 : 4000,
    });
    clearProjectMessage();
  }, [clearProjectMessage, projectMessage]);

  return null;
}
