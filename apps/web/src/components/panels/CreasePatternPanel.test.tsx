import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  createSampleProject,
  DEFAULT_CREASE_COLOR_MODE,
  type AppStatus,
  type TreeProject,
} from '../../lib/sampleProject';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { CreasePatternPanel } from './CreasePatternPanel';

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

let root: Root | null = null;
let container: HTMLDivElement | null = null;

function renderPanel(project: TreeProject, status: AppStatus) {
  const buildCreasePattern = vi.fn(async () => undefined);
  const optimizeScale = vi.fn(async () => undefined);
  useWorkspaceStore.setState(
    {
      ...useWorkspaceStore.getInitialState(),
      project,
      status,
      engineReady: true,
      optimizeScale,
      buildCreasePattern,
    },
    true
  );

  container = document.createElement('div');
  document.body.append(container);
  root = createRoot(container);
  act(() => {
    root?.render(<CreasePatternPanel />);
  });
  return { container, buildCreasePattern, optimizeScale };
}

afterEach(() => {
  if (root) {
    act(() => {
      root?.unmount();
    });
  }
  container?.remove();
  root = null;
  container = null;
  useWorkspaceStore.setState(useWorkspaceStore.getInitialState(), true);
});

describe('CreasePatternPanel', () => {
  it('shows an empty state with an enabled Build CP action after optimization', () => {
    const project = {
      ...createSampleProject(),
      creases: [],
      facets: [],
    };
    const { container, buildCreasePattern } = renderPanel(project, 'optimized');

    expect(container.textContent).toContain('No crease pattern');
    const button = container.querySelector('button');
    expect(button?.textContent).toContain('Build CP');
    expect(button?.disabled).toBe(false);

    act(() => {
      button?.click();
    });
    expect(buildCreasePattern).toHaveBeenCalledOnce();
  });

  it('shows an enabled Optimize Scale action before optimization', () => {
    const project = {
      ...createSampleProject(),
      creases: [],
      facets: [],
    };
    const { container, buildCreasePattern, optimizeScale } = renderPanel(
      project,
      'needs_optimization'
    );

    const button = container.querySelector('button');
    expect(button?.textContent).toContain('Optimize Scale');
    expect(button?.disabled).toBe(false);
    expect(button?.title).toBe('Optimize Scale');

    act(() => {
      button?.click();
    });
    expect(optimizeScale).toHaveBeenCalledOnce();
    expect(buildCreasePattern).not.toHaveBeenCalled();
  });

  it('disables the Optimize Scale action when the design has no edges', () => {
    const project = {
      ...createSampleProject(),
      edges: [],
      creases: [],
      facets: [],
    };
    const { container, optimizeScale } = renderPanel(project, 'ready');

    const button = container.querySelector('button');
    expect(button?.textContent).toContain('Optimize Scale');
    expect(button?.disabled).toBe(true);
    expect(button?.title).toBe('Add at least one tree edge before optimizing');

    act(() => {
      button?.click();
    });
    expect(optimizeScale).not.toHaveBeenCalled();
  });

  it('labels crease color controls without abbreviations when CP geometry exists', () => {
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready');

    expect(useWorkspaceStore.getState().creaseColorMode).toBe(DEFAULT_CREASE_COLOR_MODE);
    expect(container.querySelector('[aria-label="Crease pattern"]')).not.toBeNull();
    expect(container.textContent).toContain('Color by');
    expect(container.textContent).toContain('Crease roles');
    expect(container.textContent).toContain('M/V assignment');
    expect(container.innerHTML).toContain('Color by mountain, valley, flat, and border folds');
    expect(container.textContent).not.toContain('MVF');
    expect(container.textContent).not.toContain('AGRH');
    expect(container.textContent).not.toContain('No crease pattern');
  });

  it('maps the M/V assignment option to mountain and valley fold classes', () => {
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready');
    const buttons = Array.from(container.querySelectorAll('button'));
    const rolesButton = buttons.find((button) => button.textContent?.includes('Crease roles'));
    const mvButton = buttons.find((button) => button.textContent?.includes('M/V assignment'));

    expect(rolesButton?.getAttribute('aria-pressed')).toBe('true');
    expect(mvButton?.getAttribute('aria-pressed')).toBe('false');
    expect(container.querySelector('.crease--kind-hinge')).not.toBeNull();
    expect(container.querySelector('.crease--fold-valley')).toBeNull();

    act(() => {
      mvButton?.click();
    });

    expect(useWorkspaceStore.getState().creaseColorMode).toBe('mvf');
    expect(container.querySelector('.crease--fold-mountain')).not.toBeNull();
    expect(container.querySelector('.crease--fold-valley')).not.toBeNull();
    expect(container.querySelector('.crease--kind-hinge')).toBeNull();
  });
});
