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

  it('sets a theme by name and ignores unknown names', () => {
    useThemeStore.getState().setThemeByName('Dracula');
    expect(useThemeStore.getState().currentTheme.name).toBe('Dracula');

    useThemeStore.getState().setThemeByName('Not a Theme');
    expect(useThemeStore.getState().currentTheme.name).toBe('Dracula');
  });
});
