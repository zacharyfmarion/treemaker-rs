import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { DESIGN_PAPER_RECT } from '../../lib/designViewport';
import { createEmptyProject, createSampleProject, type TreeProject } from '../../lib/sampleProject';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { TooltipProvider } from '../ui/Tooltip';
import { DesignPanel } from './DesignPanel';

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

function renderPanel(project: TreeProject = createSampleProject()) {
  useWorkspaceStore.setState(
    {
      ...useWorkspaceStore.getInitialState(),
      project,
      engineReady: true,
    },
    true
  );

  container = document.createElement('div');
  document.body.append(container);
  root = createRoot(container);
  act(() => {
    root?.render(
      <TooltipProvider>
        <DesignPanel />
      </TooltipProvider>
    );
  });

  const body = container.querySelector<HTMLElement>('.design-panel__body');
  if (!body) throw new Error('Design panel body did not render');
  Object.defineProperty(body, 'clientWidth', { configurable: true, value: 900 });
  Object.defineProperty(body, 'clientHeight', { configurable: true, value: 720 });
  transformMocks.centerView.mockClear();
  transformMocks.setTransform.mockClear();
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

describe('DesignPanel', () => {
  it('shows a subtle nudge when the design tree is empty', () => {
    renderPanel(createEmptyProject());

    expect(container?.textContent).toContain('Sketch the tree behind your design');
    expect(container?.textContent).toContain('Use branches for the flaps, limbs, and features');
  });

  it('anchors the empty nudge to the paper inside the zoomable SVG canvas', () => {
    renderPanel(createEmptyProject());

    const emptyState = container?.querySelector<SVGForeignObjectElement>('foreignObject.design-empty-state');

    expect(emptyState).toBeTruthy();
    expect(emptyState?.closest('svg.design-canvas')).toBeTruthy();
    expect(emptyState?.getAttribute('x')).toBe(String(DESIGN_PAPER_RECT.x));
    expect(emptyState?.getAttribute('y')).toBe(String(DESIGN_PAPER_RECT.y));
    expect(emptyState?.getAttribute('width')).toBe(String(DESIGN_PAPER_RECT.width));
    expect(emptyState?.getAttribute('height')).toBe(String(DESIGN_PAPER_RECT.height));
  });

  it('hides the empty nudge once the design has nodes', () => {
    renderPanel();

    expect(container?.textContent).not.toContain('Sketch the tree behind your design');
  });

  it('fits the paper viewport after scale optimization requests a design fit', () => {
    renderPanel();
    const project = useWorkspaceStore.getState().project;

    act(() => {
      useWorkspaceStore.setState({
        project: { ...project, scale: 1 },
        status: 'optimized',
        designViewportFitRequestId: 1,
      });
    });

    expect(transformMocks.centerView).not.toHaveBeenCalled();
    expect(transformMocks.setTransform).toHaveBeenCalledTimes(2);
    expect(transformMocks.setTransform).toHaveBeenLastCalledWith(
      expect.any(Number),
      expect.any(Number),
      1,
      0
    );
  });

  it('does not auto-fit the paper viewport without a design fit request', () => {
    renderPanel();
    const project = useWorkspaceStore.getState().project;

    act(() => {
      useWorkspaceStore.setState({
        project: { ...project, scale: 1 },
        status: 'optimized',
      });
    });

    expect(transformMocks.centerView).not.toHaveBeenCalled();
    expect(transformMocks.setTransform).not.toHaveBeenCalled();
  });

  it('toggles mirror mode from the design toolbar when symmetry is already enabled', () => {
    renderPanel({ ...createSampleProject(), hasSymmetry: true });

    const mirrorButton = Array.from(container?.querySelectorAll<HTMLButtonElement>('button') ?? [])
      .find((button) => button.textContent?.trim() === 'Mirror Nodes');
    expect(mirrorButton).toBeTruthy();

    act(() => {
      mirrorButton?.click();
    });

    expect(useWorkspaceStore.getState().toolMode).toBe('symmetry');
    const activeMirrorButton = Array.from(container?.querySelectorAll<HTMLButtonElement>('button') ?? [])
      .find((button) => button.textContent?.trim() === 'Mirror Nodes');
    expect(activeMirrorButton?.getAttribute('aria-pressed')).toBe('true');
    expect(container?.querySelector('button[aria-label="Pair Leaves"]')).toBeNull();
  });
});
