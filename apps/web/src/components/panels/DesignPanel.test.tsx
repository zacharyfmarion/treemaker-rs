import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
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

    const mirrorButton = container?.querySelector<HTMLButtonElement>('button[aria-label="Mirror"]');
    expect(mirrorButton).toBeTruthy();

    act(() => {
      mirrorButton?.click();
    });

    expect(useWorkspaceStore.getState().toolMode).toBe('symmetry');
    expect(container?.querySelector('button[aria-label="Mirror On"]')).toBeTruthy();
  });

  it('opens a symmetry leaf preview with pair and on-axis counts', () => {
    renderPanel({
      ...createSampleProject(),
      hasSymmetry: true,
      nodes: [
        {
          id: 1,
          label: 'root',
          loc: { x: 0.5, y: 0.5 },
          isLeaf: false,
          isPinned: false,
          isConditioned: false,
        },
        {
          id: 2,
          label: 'left',
          loc: { x: 0.2, y: 0.25 },
          isLeaf: true,
          isPinned: false,
          isConditioned: false,
        },
        {
          id: 3,
          label: 'right',
          loc: { x: 0.8, y: 0.25 },
          isLeaf: true,
          isPinned: false,
          isConditioned: false,
        },
        {
          id: 4,
          label: 'axis',
          loc: { x: 0.504, y: 0.8 },
          isLeaf: true,
          isPinned: false,
          isConditioned: false,
        },
      ],
      edges: [],
      paths: [],
      conditions: [],
    });

    const pairButton = container?.querySelector<HTMLButtonElement>('button[aria-label="Pair Leaves"]');
    expect(pairButton).toBeTruthy();

    act(() => {
      pairButton?.click();
    });

    const counts = Array.from(
      container?.querySelectorAll('.symmetry-preview-popover__grid strong') ?? []
    ).map((node) => node.textContent);
    expect(counts).toEqual(['1', '1', '0', '0']);
  });
});
