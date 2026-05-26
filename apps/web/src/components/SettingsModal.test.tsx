import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useLayoutStore } from '../store/layoutStore';
import {
  DEFAULT_CAMV_ANGLE_TOLERANCE,
  useSettingsStore,
  type SettingsTab,
} from '../store/settingsStore';
import { useThemeStore } from '../store/themeStore';
import { applyTheme, DEFAULT_THEME, PRESET_THEMES } from '../themes';
import { CommandDialogModal } from './CommandDialogModal';
import { SettingsModal } from './SettingsModal';

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

const initialSettingsState = useSettingsStore.getInitialState();
const initialLayoutState = useLayoutStore.getInitialState();

let root: Root | null = null;
let container: HTMLDivElement | null = null;

function findButton(label: string): HTMLButtonElement {
  const button = Array.from(container?.querySelectorAll('button') ?? []).find((element) =>
    element.textContent?.includes(label)
  );
  expect(button).toBeDefined();
  return button as HTMLButtonElement;
}

function findExactButton(label: string): HTMLButtonElement {
  const button = Array.from(container?.querySelectorAll('button') ?? []).find(
    (element) => element.textContent === label
  );
  expect(button).toBeDefined();
  return button as HTMLButtonElement;
}

function setInputValue(input: HTMLInputElement, value: string) {
  const valueSetter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
  valueSetter?.call(input, value);
  input.dispatchEvent(new Event('input', { bubbles: true }));
}

function themeNamesForSection(rendered: HTMLElement, label: string): string[] {
  const section = Array.from(rendered.querySelectorAll('.settings-section')).find((element) =>
    element.querySelector('.settings-section__title')?.textContent?.includes(label)
  );
  expect(section).toBeDefined();
  return Array.from(section?.querySelectorAll('.settings-theme-card__name') ?? []).map(
    (element) => element.textContent ?? ''
  );
}

function renderModal(tab?: SettingsTab) {
  useSettingsStore.getState().openSettings(tab);
  container = document.createElement('div');
  document.body.append(container);
  root = createRoot(container);
  act(() => {
    root?.render(
      <>
        <SettingsModal />
        <CommandDialogModal />
      </>
    );
  });
  return container;
}

beforeEach(() => {
  localStorage.clear();
  applyTheme(DEFAULT_THEME);
  useThemeStore.setState({
    currentTheme: DEFAULT_THEME,
    presetThemes: PRESET_THEMES,
  });
  useSettingsStore.setState(initialSettingsState, true);
  useLayoutStore.setState(initialLayoutState, true);
});

afterEach(() => {
  if (root) {
    act(() => {
      root?.unmount();
    });
  }
  container?.remove();
  root = null;
  container = null;
  vi.restoreAllMocks();
});

describe('SettingsModal', () => {
  it('renders Cascade themes and applies a selected theme', () => {
    const rendered = renderModal();

    expect(rendered.querySelector('[role="dialog"]')).not.toBeNull();
    expect(rendered.textContent).toContain('Appearance');
    expect(rendered.textContent).toContain('Solarized Dark');
    expect(rendered.textContent).toContain('GitHub Light');

    expect(themeNamesForSection(rendered, 'Dark')[0]).toBe('One Dark');
    expect(themeNamesForSection(rendered, 'Light')[0]).toBe('Atom One Light');

    act(() => {
      findButton('GitHub Light').click();
    });

    expect(useThemeStore.getState().currentTheme.name).toBe('GitHub Light');
    expect(document.documentElement.getAttribute('data-theme-type')).toBe('light');
  });

  it('opens the requested tab and can reset the layout', async () => {
    const resetLayout = vi.fn();
    useLayoutStore.setState({ resetLayout });

    const rendered = renderModal('workspace');
    expect(rendered.textContent).toContain('Workspace');

    act(() => {
      findButton('Reset Layout').click();
    });

    expect(rendered.textContent).toContain('Restore the default panel layout?');

    await act(async () => {
      findExactButton('Reset').click();
      await Promise.resolve();
    });

    expect(resetLayout).toHaveBeenCalledOnce();
  });

  it('opens diagnostics settings and updates the CAMV tolerance', () => {
    const rendered = renderModal('diagnostics');
    expect(rendered.textContent).toContain('Diagnostics');

    const input = rendered.querySelector<HTMLInputElement>(
      'input[aria-label="CAMV angle tolerance"]'
    );
    expect(input).not.toBeNull();
    expect(input?.value).toBe(String(DEFAULT_CAMV_ANGLE_TOLERANCE));

    act(() => {
      if (!input) throw new Error('expected CAMV angle tolerance input');
      setInputValue(input, '0.25');
    });

    expect(useSettingsStore.getState().camvAngleTolerance).toBe(0.25);

    act(() => {
      findButton('Reset CAMV Tolerance').click();
    });

    expect(useSettingsStore.getState().camvAngleTolerance).toBe(DEFAULT_CAMV_ANGLE_TOLERANCE);
  });

  it('closes on Escape', () => {
    renderModal();

    act(() => {
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    });

    expect(useSettingsStore.getState().isSettingsOpen).toBe(false);
  });
});
