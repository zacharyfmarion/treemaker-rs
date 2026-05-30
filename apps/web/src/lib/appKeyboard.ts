import { selectionSize } from './selection';
import type { DocumentMode, Selection } from './sampleProject';
import { isShortcutEditingTarget } from '../keyboard/shortcutDispatcher';
import { handleShortcutRuntimeKeyDown } from '../keyboard/shortcutRuntime';
import type { ShortcutOverrides } from '../keyboard/shortcuts';

export interface AppKeyboardActions {
  getDocumentMode: () => DocumentMode;
  getActiveEditingSurface: () => DocumentMode;
  getCpSelectionSize: () => number;
  getSelection: () => Selection;
  handleMenuAction: (id: string) => unknown;
  selectNone: () => void;
  getShortcutOverrides?: () => ShortcutOverrides;
}

export function handleAppKeyDown(event: KeyboardEvent, actions: AppKeyboardActions): boolean {
  if (event.defaultPrevented || isShortcutEditingTarget(event.target)) return false;

  if (event.key === 'Escape') {
    if (actions.getDocumentMode() === 'crease-pattern') {
      if (actions.getCpSelectionSize() === 0) return false;
      event.preventDefault();
      void actions.handleMenuAction('edit.deselectAll');
      return true;
    }
    if (selectionSize(actions.getSelection()) === 0) return false;
    event.preventDefault();
    actions.selectNone();
    return true;
  }

  return handleShortcutRuntimeKeyDown(event, {
    context: {
      documentMode: actions.getDocumentMode(),
      activeEditingSurface: actions.getActiveEditingSurface(),
    },
    overrides: actions.getShortcutOverrides?.(),
    menu: actions.handleMenuAction,
  });
}

export function installAppKeyboardListener(
  actions: AppKeyboardActions,
  target: Document = document
): () => void {
  const onKeyDown = (event: KeyboardEvent) => {
    handleAppKeyDown(event, actions);
  };
  target.addEventListener('keydown', onKeyDown, true);
  return () => target.removeEventListener('keydown', onKeyDown, true);
}
