import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import { applyTheme, DEFAULT_THEME, PRESET_THEMES, type TreeMakerTheme } from '../themes';

export const THEME_STORAGE_KEY = 'treemaker-web-theme';

function loadSavedThemeName(): string | null {
  if (typeof localStorage === 'undefined') return null;
  try {
    return localStorage.getItem(THEME_STORAGE_KEY);
  } catch {
    return null;
  }
}

function saveThemeName(name: string): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(THEME_STORAGE_KEY, name);
  } catch {
    // Ignore storage failures in restricted browser contexts.
  }
}

function resolveInitialTheme(): TreeMakerTheme {
  const savedName = loadSavedThemeName();
  if (!savedName) return DEFAULT_THEME;
  return PRESET_THEMES.find((theme) => theme.name === savedName) ?? DEFAULT_THEME;
}

interface ThemeState {
  currentTheme: TreeMakerTheme;
  presetThemes: TreeMakerTheme[];
  setTheme: (theme: TreeMakerTheme) => void;
  setThemeByName: (name: string) => void;
}

export const useThemeStore = create<ThemeState>()(
  devtools(
    (set, get) => {
      const initialTheme = resolveInitialTheme();
      applyTheme(initialTheme);

      return {
        currentTheme: initialTheme,
        presetThemes: PRESET_THEMES,

        setTheme: (theme) => {
          applyTheme(theme);
          saveThemeName(theme.name);
          set({ currentTheme: theme });
        },

        setThemeByName: (name) => {
          const theme = get().presetThemes.find((preset) => preset.name === name);
          if (theme) get().setTheme(theme);
        },
      };
    },
    { name: 'ThemeStore' }
  )
);
