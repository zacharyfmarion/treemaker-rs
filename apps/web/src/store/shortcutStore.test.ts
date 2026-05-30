import { beforeEach, describe, expect, it } from 'vitest';
import { getResolvedShortcuts } from '../keyboard/shortcuts';
import { SHORTCUT_STORAGE_KEY, useShortcutStore } from './shortcutStore';

const initialState = useShortcutStore.getInitialState();

beforeEach(() => {
  localStorage.clear();
  useShortcutStore.setState(initialState, true);
});

describe('shortcutStore', () => {
  it('persists overrides and disabled shortcuts', () => {
    useShortcutStore.getState().setShortcut('file.save', { primary: true, alt: true, key: 's' });
    useShortcutStore.getState().clearShortcut('cp.action.line-type.mountain');

    expect(useShortcutStore.getState().overrides['file.save']).toEqual([
      {
        primary: true,
        alt: true,
        key: 's',
      },
    ]);
    expect(useShortcutStore.getState().overrides['cp.action.line-type.mountain']).toBeNull();

    const stored = JSON.parse(localStorage.getItem(SHORTCUT_STORAGE_KEY) ?? '{}') as {
      bindings?: Record<string, unknown>;
    };
    expect(stored.bindings?.['file.save']).toEqual([{ primary: true, alt: true, key: 's' }]);
    expect(stored.bindings?.['cp.action.line-type.mountain']).toBeNull();
  });

  it('resets individual shortcuts and all shortcuts', () => {
    useShortcutStore.getState().setShortcut('file.save', { primary: true, alt: true, key: 's' });
    useShortcutStore.getState().resetShortcut('file.save');

    expect(useShortcutStore.getState().overrides['file.save']).toBeUndefined();

    useShortcutStore.getState().setShortcut('file.open', { primary: true, alt: true, key: 'o' });
    useShortcutStore.getState().resetAllShortcuts();

    expect(useShortcutStore.getState().overrides).toEqual({});
  });

  it('keeps protected undo defaults when clearing custom bindings', () => {
    useShortcutStore.getState().setShortcut('edit.undo', {
      primary: true,
      alt: true,
      key: 'z',
    });
    expect(getResolvedShortcuts('edit.undo', useShortcutStore.getState().overrides)).toEqual([
      { primary: true, key: 'z' },
      { primary: true, alt: true, key: 'z' },
    ]);

    useShortcutStore.getState().clearShortcut('edit.undo');

    expect(useShortcutStore.getState().overrides['edit.undo']).toBeUndefined();
    expect(getResolvedShortcuts('edit.undo', useShortcutStore.getState().overrides)).toEqual([
      { primary: true, key: 'z' },
    ]);
  });
});
