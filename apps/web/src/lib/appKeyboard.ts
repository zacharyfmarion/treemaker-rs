import { selectionSize } from './selection';
import type { DocumentMode, Selection } from './sampleProject';
import {
  handleShortcutKeyDown,
  isShortcutEditingTarget,
} from '../keyboard/shortcutDispatcher';
import type { ShortcutOverrides } from '../keyboard/shortcuts';

interface AppKeyboardActions {
  getDocumentMode: () => DocumentMode;
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

  return handleShortcutKeyDown(event, {
    scopeStack: ['global'],
    overrides: actions.getShortcutOverrides?.(),
    executors: {
      menu: actions.handleMenuAction,
    },
  });
}
