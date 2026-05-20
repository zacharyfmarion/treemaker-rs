import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  createSampleProject,
  DEFAULT_CREASE_COLOR_MODE,
  type AppStatus,
  type TreeProject,
} from '../../lib/sampleProject';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { TooltipProvider } from '../ui/Tooltip';
import { CreasePatternPanel } from './CreasePatternPanel';

const transformMocks = vi.hoisted(() => ({
  centerView: vi.fn(),
  setTransform: vi.fn(),
  zoomIn: vi.fn(),
  zoomOut: vi.fn(),
}));

vi.mock('react-zoom-pan-pinch', async () => {
  const React = await import('react');
  type MockTransformWrapperProps = {
    children: React.ReactNode;
    onInit?: (ref: unknown) => void;
    onTransformed?: (ref: unknown, state: { scale: number }) => void;
  };
  const api = {
    centerView: transformMocks.centerView,
    setTransform: transformMocks.setTransform,
    zoomIn: transformMocks.zoomIn,
    zoomOut: transformMocks.zoomOut,
  };

  return {
    TransformWrapper: React.forwardRef<unknown, MockTransformWrapperProps>(
      function MockTransformWrapper({ children, onInit, onTransformed }, ref) {
        const didInitRef = React.useRef(false);
        React.useImperativeHandle(ref, () => api, []);
        React.useEffect(() => {
          if (didInitRef.current) return;
          didInitRef.current = true;
          onInit?.(api);
          onTransformed?.(api, { scale: 1 });
        }, [onInit, onTransformed]);
        return React.createElement('div', { 'data-testid': 'transform-wrapper' }, children);
      }
    ),
    TransformComponent: ({ children }: { children: React.ReactNode }) =>
      React.createElement('div', { 'data-testid': 'transform-component' }, children),
  };
});

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

let root: Root | null = null;
let container: HTMLDivElement | null = null;

beforeEach(() => {
  vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => {
    callback(0);
    return 1;
  });
  vi.stubGlobal('cancelAnimationFrame', vi.fn());
});

function renderPanel(
  project: TreeProject,
  status: AppStatus,
  state: Partial<ReturnType<typeof useWorkspaceStore.getState>> = {}
) {
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
      ...state,
    },
    true
  );

  container = document.createElement('div');
  document.body.append(container);
  root = createRoot(container);
  act(() => {
    root?.render(
      <TooltipProvider>
        <CreasePatternPanel />
      </TooltipProvider>
    );
  });
  const body = container.querySelector<HTMLElement>('.cp-panel__body');
  if (!body) throw new Error('Crease pattern panel body did not render');
  Object.defineProperty(body, 'clientWidth', { configurable: true, value: 900 });
  Object.defineProperty(body, 'clientHeight', { configurable: true, value: 720 });
  transformMocks.centerView.mockClear();
  transformMocks.setTransform.mockClear();
  transformMocks.zoomIn.mockClear();
  transformMocks.zoomOut.mockClear();
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
  transformMocks.centerView.mockClear();
  transformMocks.setTransform.mockClear();
  transformMocks.zoomIn.mockClear();
  transformMocks.zoomOut.mockClear();
  vi.unstubAllGlobals();
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

  it('keeps an empty CP-ready tree on Build CP instead of Rebuild CP', () => {
    const project = {
      ...createSampleProject(),
      creases: [],
      facets: [],
    };
    const { container } = renderPanel(project, 'crease_pattern_ready');

    const button = container.querySelector('button');
    expect(button?.textContent).toContain('Build CP');
    expect(button?.textContent).not.toContain('Rebuild CP');
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

  it('shows CP viewport controls and wires zoom, fit, and preset actions', () => {
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready');

    expect(container.querySelector('[aria-label="Crease pattern viewport controls"]')).not.toBeNull();

    act(() => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Zoom In"]')?.click();
    });
    expect(transformMocks.zoomIn).toHaveBeenCalledWith(0.35, 120);

    act(() => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Zoom Out"]')?.click();
    });
    expect(transformMocks.zoomOut).toHaveBeenCalledWith(0.35, 120);

    act(() => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Fit"]')?.click();
    });
    expect(transformMocks.centerView).toHaveBeenLastCalledWith(expect.any(Number), 180);

    act(() => {
      Array.from(container.querySelectorAll<HTMLButtonElement>('button'))
        .find((button) => button.textContent?.trim() === '1:1')
        ?.click();
    });
    expect(transformMocks.centerView).toHaveBeenLastCalledWith(1, 160);

    act(() => {
      container.querySelector<HTMLButtonElement>('.viewport-toolbar__zoom-button')?.click();
    });
    act(() => {
      Array.from(container.querySelectorAll<HTMLButtonElement>('.viewport-toolbar__dropdown-item'))
        .find((button) => button.textContent?.trim() === '200%')
        ?.click();
    });
    expect(transformMocks.centerView).toHaveBeenLastCalledWith(2, 160);
  });

  it('supports the same CP viewport keyboard shortcuts and space-pan marker', () => {
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready');
    const body = container.querySelector<HTMLElement>('.cp-panel__body');
    expect(body).not.toBeNull();

    act(() => {
      body?.dispatchEvent(new KeyboardEvent('keydown', { key: '=', metaKey: true, bubbles: true }));
    });
    expect(transformMocks.zoomIn).toHaveBeenCalledWith(0.35, 120);

    act(() => {
      body?.dispatchEvent(new KeyboardEvent('keydown', { key: '-', metaKey: true, bubbles: true }));
    });
    expect(transformMocks.zoomOut).toHaveBeenCalledWith(0.35, 120);

    act(() => {
      body?.dispatchEvent(new KeyboardEvent('keydown', { key: '0', metaKey: true, bubbles: true }));
    });
    expect(transformMocks.centerView).toHaveBeenLastCalledWith(expect.any(Number), 180);

    act(() => {
      body?.dispatchEvent(new KeyboardEvent('keydown', { key: '1', metaKey: true, bubbles: true }));
    });
    expect(transformMocks.centerView).toHaveBeenLastCalledWith(1, 160);

    act(() => {
      body?.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }));
    });
    expect(body?.getAttribute('data-space-pan')).toBe('true');

    act(() => {
      body?.dispatchEvent(new KeyboardEvent('keyup', { key: ' ', bubbles: true }));
    });
    expect(body?.hasAttribute('data-space-pan')).toBe(false);
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

  it('clears crease-pattern selection when the user clicks the canvas background', () => {
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready');

    act(() => {
      useWorkspaceStore.getState().selectAll();
    });
    expect(useWorkspaceStore.getState().selection.kind).toBe('multi');

    const canvas = container.querySelector<SVGSVGElement>('.cp-canvas');
    expect(canvas).toBeTruthy();

    act(() => {
      canvas?.dispatchEvent(new MouseEvent('pointerdown', { bubbles: true, button: 0 }));
    });

    expect(useWorkspaceStore.getState().selection).toEqual({ kind: 'tree' });
  });

  it('does not show tree workflow actions for imported CP-only empty states', () => {
    const project = {
      ...createSampleProject(),
      edges: [],
      creases: [],
      facets: [],
    };
    const { container } = renderPanel(project, 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: null,
    });

    expect(container.textContent).toContain('No imported crease pattern');
    expect(container.textContent).not.toContain('Optimize Scale');
    expect(container.textContent).not.toContain('Build CP');
  });
});
