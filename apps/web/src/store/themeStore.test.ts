import { beforeEach, describe, expect, it } from 'vitest';
import { applyTheme, DEFAULT_THEME, PRESET_THEMES } from '../themes';
import { THEME_STORAGE_KEY, useThemeStore } from './themeStore';

beforeEach(() => {
  localStorage.clear();
  applyTheme(DEFAULT_THEME);
  useThemeStore.setState({
    currentTheme: DEFAULT_THEME,
    presetThemes: PRESET_THEMES,
  });
});

describe('themeStore', () => {
  it('exposes the Cascade preset theme set', () => {
    const names = useThemeStore.getState().presetThemes.map((theme) => theme.name);

    expect(names).toHaveLength(23);
    expect(names).toContain('Solarized Dark');
    expect(names).toContain('GitHub Light');
    expect(names).toContain('Shades of Purple');
  });

  it('applies and persists the selected theme', () => {
    const theme = PRESET_THEMES.find((preset) => preset.name === 'GitHub Light');
    expect(theme).toBeDefined();

    useThemeStore.getState().setTheme(theme ?? DEFAULT_THEME);

    expect(useThemeStore.getState().currentTheme.name).toBe('GitHub Light');
    expect(localStorage.getItem(THEME_STORAGE_KEY)).toBe('GitHub Light');
    expect(document.documentElement.style.getPropertyValue('--bg-primary')).toBe(
      theme?.colors['bg.primary']
    );
    expect(document.documentElement.getAttribute('data-theme-type')).toBe('light');
  });

  it('keeps M/V assignment colors semantic instead of theme-accent driven', () => {
    const lightTheme = PRESET_THEMES.find((preset) => preset.name === 'GitHub Light');
    const darkTheme = PRESET_THEMES.find((preset) => preset.name === 'Solarized Dark');
    expect(lightTheme).toBeDefined();
    expect(darkTheme).toBeDefined();

    useThemeStore.getState().setTheme(lightTheme ?? DEFAULT_THEME);
    expect(document.documentElement.style.getPropertyValue('--fold-mountain')).toBe('#d91f3a');
    expect(document.documentElement.style.getPropertyValue('--fold-valley')).toBe('#2563eb');

    useThemeStore.getState().setTheme(darkTheme ?? DEFAULT_THEME);
    expect(document.documentElement.style.getPropertyValue('--fold-mountain')).toBe('#ff4d5d');
    expect(document.documentElement.style.getPropertyValue('--fold-valley')).toBe('#60a5fa');
  });

  it('sets a theme by name and ignores unknown names', () => {
    useThemeStore.getState().setThemeByName('Dracula');
    expect(useThemeStore.getState().currentTheme.name).toBe('Dracula');

    useThemeStore.getState().setThemeByName('Not a Theme');
    expect(useThemeStore.getState().currentTheme.name).toBe('Dracula');
  });
});
