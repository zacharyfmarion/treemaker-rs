import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import {
  normalizeKeyChord,
  type KeyChord,
  type ShortcutActionId,
  type ShortcutOverrides,
} from '../keyboard/shortcuts';

export const SHORTCUT_STORAGE_KEY = 'oristudio-shortcuts-v1';

interface PersistedShortcutState {
  version: 1;
  bindings: Record<string, KeyChord | KeyChord[] | null>;
}

interface ShortcutState {
  overrides: ShortcutOverrides;
  setShortcut: (id: ShortcutActionId, chord: KeyChord) => void;
  clearShortcut: (id: ShortcutActionId) => void;
  resetShortcut: (id: ShortcutActionId) => void;
  resetAllShortcuts: () => void;
}

function loadShortcutOverrides(): ShortcutOverrides {
  if (typeof localStorage === 'undefined') return {};
  try {
    const raw = localStorage.getItem(SHORTCUT_STORAGE_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as Partial<PersistedShortcutState>;
    if (parsed.version !== 1 || !parsed.bindings || typeof parsed.bindings !== 'object') {
      return {};
    }
    const overrides: ShortcutOverrides = {};
    for (const [id, binding] of Object.entries(parsed.bindings)) {
      if (binding === null) {
        overrides[id as ShortcutActionId] = null;
        continue;
      }
      const chords = Array.isArray(binding) ? binding : [binding];
      overrides[id as ShortcutActionId] = chords
        .map((chord) => normalizeKeyChord(chord))
        .filter((chord) => chord.key);
    }
    return overrides;
  } catch {
    return {};
  }
}

function saveShortcutOverrides(overrides: ShortcutOverrides): void {
  if (typeof localStorage === 'undefined') return;
  try {
    const persisted: PersistedShortcutState = {
      version: 1,
      bindings: Object.fromEntries(
        Object.entries(overrides).filter(
          (entry): entry is [string, KeyChord[] | null] => entry[1] !== undefined
        )
      ),
    };
    localStorage.setItem(SHORTCUT_STORAGE_KEY, JSON.stringify(persisted));
  } catch {
    // Ignore storage failures in restricted browser contexts.
  }
}

export const useShortcutStore = create<ShortcutState>()(
  devtools(
    (set, get) => ({
      overrides: loadShortcutOverrides(),

      setShortcut: (id, chord) => {
        const overrides = { ...get().overrides, [id]: [normalizeKeyChord(chord)] };
        saveShortcutOverrides(overrides);
        set({ overrides });
      },

      clearShortcut: (id) => {
        const overrides = { ...get().overrides, [id]: null };
        saveShortcutOverrides(overrides);
        set({ overrides });
      },

      resetShortcut: (id) => {
        const overrides = { ...get().overrides };
        delete overrides[id];
        saveShortcutOverrides(overrides);
        set({ overrides });
      },

      resetAllShortcuts: () => {
        saveShortcutOverrides({});
        set({ overrides: {} });
      },
    }),
    { name: 'ShortcutStore' }
  )
);
