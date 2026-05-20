import { selectionSize } from './selection';
import type { Selection } from './sampleProject';

interface AppKeyboardActions {
  deleteSelection: () => unknown;
  getSelection: () => Selection;
  handleMenuAction: (id: string) => unknown;
  selectNone: () => void;
}

function isEditingTarget(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement
  );
}

export function handleAppKeyDown(event: KeyboardEvent, actions: AppKeyboardActions): boolean {
  if (event.defaultPrevented || isEditingTarget(event.target)) return false;

  if (event.key === 'Escape') {
    if (selectionSize(actions.getSelection()) === 0) return false;
    event.preventDefault();
    actions.selectNone();
    return true;
  }

  const modifier = event.metaKey || event.ctrlKey;
  const key = event.key.toLowerCase();
  if (modifier && key === 'z') {
    event.preventDefault();
    void actions.handleMenuAction(event.shiftKey ? 'edit.redo' : 'edit.undo');
    return true;
  }
  if (modifier && key === 'x') {
    event.preventDefault();
    void actions.handleMenuAction('edit.cut');
    return true;
  }
  if (modifier && key === 'c') {
    event.preventDefault();
    void actions.handleMenuAction('edit.copy');
    return true;
  }
  if (modifier && key === 'v') {
    event.preventDefault();
    void actions.handleMenuAction('edit.paste');
    return true;
  }
  if (modifier && key === 'a') {
    event.preventDefault();
    void actions.handleMenuAction('edit.selectAll');
    return true;
  }
  if (modifier && key === ',') {
    event.preventDefault();
    void actions.handleMenuAction('file.settings');
    return true;
  }
  if (event.key === 'F1') {
    event.preventDefault();
    void actions.handleMenuAction('help.documentation');
    return true;
  }
  if (event.key !== 'Delete' && event.key !== 'Backspace') return false;
  event.preventDefault();
  void actions.deleteSelection();
  return true;
}
