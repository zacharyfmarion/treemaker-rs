import { beforeEach, describe, expect, it } from 'vitest';
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
});
