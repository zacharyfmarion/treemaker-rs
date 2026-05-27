import {
  getResolvedShortcuts,
  keyChordEquals,
  keyChordFromKeyboardEvent,
  SHORTCUT_DEFINITIONS,
  type ShortcutActionId,
  type ShortcutOverrides,
  type ShortcutScope,
  type ViewportShortcutId,
} from './shortcuts';
import type { MenuActionId } from '../commands/menuActions';
import type { OristudioCpActionId } from '../lib/oristudioCpActions';

export interface ShortcutExecutors {
  menu?: (id: MenuActionId) => unknown;
  cpAction?: (id: OristudioCpActionId) => unknown;
  viewport?: (id: ViewportShortcutId) => unknown;
}

export interface ShortcutDispatchOptions {
  scopeStack: ShortcutScope[];
  overrides?: ShortcutOverrides;
  executors: ShortcutExecutors;
}

export function isShortcutEditingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  return (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement ||
    target.isContentEditable
  );
}

export function handleShortcutKeyDown(
  event: KeyboardEvent,
  options: ShortcutDispatchOptions
): boolean {
  if (event.defaultPrevented || event.isComposing || isShortcutEditingTarget(event.target)) {
    return false;
  }

  const chord = keyChordFromKeyboardEvent(event);
  if (!chord) return false;

  for (const scope of options.scopeStack) {
    const definition = SHORTCUT_DEFINITIONS.find((candidate) => {
      if (candidate.scope !== scope) return false;
      return getResolvedShortcuts(candidate.id, options.overrides).some((shortcut) =>
        keyChordEquals(shortcut, chord)
      );
    });

    if (!definition) continue;
    if (!executeShortcut(definition.id, definition.target, options.executors)) return false;
    event.preventDefault();
    return true;
  }

  return false;
}

function executeShortcut(
  id: ShortcutActionId,
  target: 'menu' | 'cp-action' | 'viewport',
  executors: ShortcutExecutors
): boolean {
  switch (target) {
    case 'menu':
      if (!executors.menu) return false;
      void executors.menu(id as MenuActionId);
      return true;
    case 'cp-action':
      if (!executors.cpAction) return false;
      void executors.cpAction(id as OristudioCpActionId);
      return true;
    case 'viewport':
      if (!executors.viewport) return false;
      void executors.viewport(id as ViewportShortcutId);
      return true;
  }
}
