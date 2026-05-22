import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  createSampleProject,
  DEFAULT_CREASE_COLOR_MODE,
  type AppStatus,
  type TreeProject,
} from '../../lib/sampleProject';
import type { ImportedCreasePatternDocument } from '../../lib/creasePatternImport';
import type {
  OristudioCpCommandPayload,
  OristudioCpDocumentState,
} from '../../engine/oristudioCpTypes';
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

function importedCpDocument(): ImportedCreasePatternDocument {
  return {
    source: { format: 'cp', filename: 'editable.cp', path: null },
    title: 'editable',
    selectedFrame: null,
    fold: {
      file_spec: 1.2,
      file_creator: 'test',
      frame_title: 'editable',
      vertices_coords: [
        [0, 0],
        [1, 0],
      ],
      edges_vertices: [[0, 1]],
      edges_assignment: ['B'],
      faces_vertices: [],
    },
    lineOnly: true,
    simulationModelError: null,
    diagnostics: { warnings: [], errors: [] },
    stats: {
      vertices: 2,
      edges: 1,
      faces: 0,
      mountains: 0,
      valleys: 0,
      boundaries: 1,
      flats: 0,
      unassigned: 0,
    },
  };
}

function editableCpState(): OristudioCpDocumentState {
  return {
    handle: 1,
    source: { format: 'cp', filename: 'editable.cp', path: null },
    operationDescriptors: [],
    lastCommandResult: null,
    summary: {
      title: 'editable',
      line_segments: 2,
      circles: 1,
      points: 1,
      aux_line_segments: 0,
      texts: 1,
      can_save_as_cp: false,
      is_empty: false,
    },
    document: {
      title: 'editable',
      metadata: {},
      crease_pattern: {
        line_segments: [
          {
            a: { x: 0, y: 0 },
            b: { x: 1, y: 0 },
            active: 'Inactive0',
            color: 'Red1',
            selected: 0,
            customized: 0,
            customized_color: { red: 100, green: 200, blue: 200 },
          },
          {
            a: { x: 0, y: 0 },
            b: { x: 0, y: 1 },
            active: 'Inactive0',
            color: 'Blue2',
            selected: 0,
            customized: 0,
            customized_color: { red: 100, green: 200, blue: 200 },
          },
        ],
        circles: [
          {
            x: 0.5,
            y: 0.5,
            r: 0.25,
            color: 'Cyan3',
            customized: 0,
            customized_color: { red: 100, green: 200, blue: 200 },
          },
        ],
        points: [{ x: 0.5, y: 0.5 }],
        aux_line_segments: [],
        texts: [{ x: 0.2, y: 0.8, text: 'note' }],
        grid: {
          interval_grid_size: 2,
          grid_size: 8,
          grid_xa: 1,
          grid_xb: 0,
          grid_xc: 1,
          grid_ya: 1,
          grid_yb: 0,
          grid_yc: 1,
          grid_angle: 90,
          base_state: 'WithinPaper',
          vertical_scale_position: 0,
          horizontal_scale_position: 0,
          draw_diagonal_gridlines: false,
        },
      },
    },
  };
}

function setCanvasClientRect(container: HTMLElement): SVGSVGElement {
  const canvas = container.querySelector<SVGSVGElement>('.cp-canvas');
  if (!canvas) throw new Error('expected CP canvas');
  Object.defineProperty(canvas, 'getBoundingClientRect', {
    configurable: true,
    value: () =>
      ({
        x: 0,
        y: 0,
        left: 0,
        top: 0,
        right: 720,
        bottom: 720,
        width: 720,
        height: 720,
        toJSON: () => ({}),
      }) as DOMRect,
  });
  return canvas;
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
    expect(container.querySelector('.cp-tool-rail')).toBeNull();
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

  it('renders editable CP kernel geometry with grid, selection, and viewport toggles', () => {
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
    });

    expect(container.textContent).toContain('Editable kernel: 2 lines');
    expect(container.querySelectorAll('[data-cp-line-id]')).toHaveLength(2);
    expect(container.querySelectorAll('[data-cp-line-hit-id]')).toHaveLength(2);
    expect(container.querySelectorAll('[data-cp-vertex-id]')).toHaveLength(3);
    expect(container.querySelector('.cp-grid-line')).not.toBeNull();
    expect(container.querySelector('.cp-circle')).not.toBeNull();
    expect(container.querySelector('.cp-point')).not.toBeNull();
    expect(container.querySelector('.cp-text')?.textContent).toBe('note');
    expect(container.querySelector('.cp-tool-rail')).not.toBeNull();
    expect(container.textContent).toContain('Tool Select');
    expect(container.textContent).toContain('2 lines');

    const drawCreaseButton = container.querySelector<HTMLButtonElement>(
      'button[aria-label="Draw crease"]'
    );
    const foldEstimateButton = container.querySelector<HTMLButtonElement>(
      'button[aria-label="Fold estimate"]'
    );
    const makeMountainButton = container.querySelector<HTMLButtonElement>(
      'button[aria-label="Make mountain"]'
    );
    const moveButton = container.querySelector<HTMLButtonElement>(
      'button[aria-label="Move selected creases"]'
    );
    const deleteIntersectingButton = container.querySelector<HTMLButtonElement>(
      'button[aria-label="Delete intersecting creases"]'
    );
    const selectIntersectingButton = container.querySelector<HTMLButtonElement>(
      'button[aria-label="Select intersecting line"]'
    );
    const fixInaccurateButton = container.querySelector<HTMLButtonElement>(
      'button[aria-label="Fix inaccurate creases"]'
    );
    const operationFrameButton = container.querySelector<HTMLButtonElement>(
      'button[aria-label="Operation frame"]'
    );
    const selectLassoButton = container.querySelector<HTMLButtonElement>(
      'button[aria-label="Select lasso"]'
    );
    expect(drawCreaseButton?.getAttribute('aria-disabled')).toBe('true');
    expect(drawCreaseButton?.getAttribute('data-ui-status')).toBe('not-implemented');
    expect(makeMountainButton?.getAttribute('aria-disabled')).toBe('false');
    expect(makeMountainButton?.getAttribute('data-ui-status')).toBe('ready');
    expect(moveButton?.getAttribute('aria-disabled')).toBe('false');
    expect(moveButton?.getAttribute('data-ui-status')).toBe('ready');
    expect(deleteIntersectingButton?.getAttribute('aria-disabled')).toBe('false');
    expect(deleteIntersectingButton?.getAttribute('data-ui-status')).toBe('ready');
    expect(selectIntersectingButton?.getAttribute('aria-disabled')).toBe('false');
    expect(selectIntersectingButton?.getAttribute('data-ui-status')).toBe('ready');
    expect(fixInaccurateButton?.getAttribute('aria-disabled')).toBe('false');
    expect(fixInaccurateButton?.getAttribute('data-ui-status')).toBe('ready');
    expect(operationFrameButton?.getAttribute('aria-disabled')).toBe('false');
    expect(operationFrameButton?.getAttribute('data-ui-status')).toBe('ready');
    expect(selectLassoButton?.getAttribute('aria-disabled')).toBe('false');
    expect(selectLassoButton?.getAttribute('data-ui-status')).toBe('ready');
    expect(foldEstimateButton?.getAttribute('aria-disabled')).toBe('true');
    expect(foldEstimateButton?.getAttribute('data-ui-status')).toBe('porting');

    act(() => {
      drawCreaseButton?.click();
    });
    expect(container.textContent).toContain('Draw crease: Not implemented');
    expect(drawCreaseButton?.hasAttribute('data-active')).toBe(true);

    act(() => {
      foldEstimateButton?.click();
    });
    expect(container.textContent).toContain('Fold estimate: Porting');
    expect(foldEstimateButton?.hasAttribute('data-active')).toBe(true);

    act(() => {
      container.querySelector<SVGLineElement>('[data-cp-line-id="1"]')?.dispatchEvent(
        new MouseEvent('click', { bubbles: true })
      );
    });

    expect(useWorkspaceStore.getState().oristudioCpSelection.lines).toEqual([1]);
    expect(container.textContent).toContain('1 selected');

    act(() => {
      container.querySelector<SVGCircleElement>('[data-cp-vertex-id="0:0"]')?.dispatchEvent(
        new MouseEvent('click', { bubbles: true, shiftKey: true })
      );
    });
    expect(useWorkspaceStore.getState().oristudioCpSelection.vertices).toEqual(['0:0']);
    expect(container.textContent).toContain('2 selected');

    const body = container.querySelector<HTMLElement>('.cp-panel__body');
    act(() => {
      body?.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true })
      );
    });
    expect(container.textContent).toContain('Tool Select');
    expect(useWorkspaceStore.getState().oristudioCpSelection.lines).toEqual([1]);

    act(() => {
      body?.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true })
      );
    });
    expect(useWorkspaceStore.getState().oristudioCpSelection.lines).toEqual([]);

    act(() => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Grid"]')?.click();
    });
    expect(useWorkspaceStore.getState().oristudioCpViewport.gridVisible).toBe(false);

    act(() => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Snap"]')?.click();
    });
    expect(useWorkspaceStore.getState().oristudioCpViewport.snapToGrid).toBe(false);
    expect(useWorkspaceStore.getState().oristudioCpViewport.snapToVertices).toBe(false);
    expect(useWorkspaceStore.getState().oristudioCpViewport.snapToLines).toBe(false);
  });

  it('runs ready CP line commands with the current editable selection payload', async () => {
    const executeOristudioCpCommand = vi.fn(async () => true);
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
      executeOristudioCpCommand,
    });

    act(() => {
      container.querySelector<SVGLineElement>('[data-cp-line-hit-id="1"]')?.dispatchEvent(
        new MouseEvent('click', { bubbles: true })
      );
    });

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Make mountain"]')?.click();
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenCalledWith('CreaseMakeMountain', {
      line_ids: [1],
    });
  });

  it('runs ready multi-step CP transform commands with resolved model points', async () => {
    const executeOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => true
    );
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
      oristudioCpViewport: {
        gridVisible: true,
        snapToGrid: false,
        snapToVertices: false,
        snapToLines: false,
      },
      executeOristudioCpCommand,
    });
    const canvas = setCanvasClientRect(container);

    act(() => {
      container.querySelector<SVGLineElement>('[data-cp-line-id="1"]')?.dispatchEvent(
        new MouseEvent('click', { bubbles: true })
      );
    });

    await act(async () => {
      container
        .querySelector<HTMLButtonElement>('button[aria-label="Move selected creases"]')
        ?.click();
      await Promise.resolve();
    });
    expect(container.textContent).toContain('Move selected creases: Pick source point');

    act(() => {
      canvas.dispatchEvent(
        new MouseEvent('pointerdown', {
          bubbles: true,
          button: 0,
          clientX: 360,
          clientY: 348,
        })
      );
    });
    expect(container.textContent).toContain('Move selected creases: Pick destination point');

    await act(async () => {
      canvas.dispatchEvent(
        new MouseEvent('pointerdown', {
          bubbles: true,
          button: 0,
          clientX: 360,
          clientY: 230.4,
        })
      );
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenCalledOnce();
    const [operation, payload] = executeOristudioCpCommand.mock.calls[0] ?? [];
    const points = payload?.points ?? [];
    expect(operation).toBe('CreaseMove');
    expect(payload?.line_ids).toEqual([1]);
    expect(points[0].x).toBeCloseTo(0);
    expect(points[0].y).toBeCloseTo(0);
    expect(points[1].x).toBeCloseTo(0);
    expect(points[1].y).toBeCloseTo(80);
  });

  it('runs ready drag-line CP delete commands without requiring selected lines', async () => {
    const executeOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => true
    );
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
      oristudioCpViewport: {
        gridVisible: true,
        snapToGrid: false,
        snapToVertices: false,
        snapToLines: false,
      },
      executeOristudioCpCommand,
    });
    const canvas = setCanvasClientRect(container);

    await act(async () => {
      container
        .querySelector<HTMLButtonElement>('button[aria-label="Delete intersecting creases"]')
        ?.click();
      await Promise.resolve();
    });
    expect(container.textContent).toContain(
      'Delete intersecting creases: Pick drag start point'
    );

    act(() => {
      canvas.dispatchEvent(
        new MouseEvent('pointerdown', {
          bubbles: true,
          button: 0,
          clientX: 360,
          clientY: 348,
        })
      );
    });

    await act(async () => {
      canvas.dispatchEvent(
        new MouseEvent('pointerdown', {
          bubbles: true,
          button: 0,
          clientX: 477.6,
          clientY: 348,
        })
      );
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenCalledOnce();
    const [operation, payload] = executeOristudioCpCommand.mock.calls[0] ?? [];
    const points = payload?.points ?? [];
    expect(operation).toBe('CreaseDeleteIntersecting');
    expect(payload?.line_ids).toEqual([]);
    expect(points[0].x).toBeCloseTo(0);
    expect(points[0].y).toBeCloseTo(0);
    expect(points[1].x).toBeCloseTo(80);
    expect(points[1].y).toBeCloseTo(0);
  });

  it('passes Oriedita line-type defaults for ready selected-type commands', async () => {
    const executeOristudioCpCommand = vi.fn(async () => true);
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
      executeOristudioCpCommand,
    });

    act(() => {
      container.querySelector<SVGLineElement>('[data-cp-line-id="1"]')?.dispatchEvent(
        new MouseEvent('click', { bubbles: true })
      );
    });

    await act(async () => {
      container
        .querySelector<HTMLButtonElement>('button[aria-label="Replace selected line type"]')
        ?.click();
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenCalledWith('ReplaceLineTypeSelect', {
      line_ids: [1],
      custom_from_line_type: 'Any',
      custom_to_line_type: 'Edge',
    });
  });

  it('runs ready lengthen CP commands with three resolved model points and current color', async () => {
    const executeOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => true
    );
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
      oristudioCpViewport: {
        gridVisible: true,
        snapToGrid: false,
        snapToVertices: false,
        snapToLines: false,
      },
      executeOristudioCpCommand,
    });
    const canvas = setCanvasClientRect(container);

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Lengthen crease"]')?.click();
      await Promise.resolve();
    });

    for (const [clientX, clientY] of [
      [360, 348],
      [477.6, 348],
      [477.6, 230.4],
    ]) {
      await act(async () => {
        canvas.dispatchEvent(
          new MouseEvent('pointerdown', {
            bubbles: true,
            button: 0,
            clientX,
            clientY,
          })
        );
        await Promise.resolve();
      });
    }

    expect(executeOristudioCpCommand).toHaveBeenCalledOnce();
    const [operation, payload] = executeOristudioCpCommand.mock.calls[0] ?? [];
    expect(operation).toBe('LengthenCrease');
    expect(payload?.points).toHaveLength(3);
    expect(payload?.line_color).toBe('Red1');
    expect(payload?.selection_distance).toBeGreaterThan(0);
  });

  it('runs ready lasso commands from a freehand drag path', async () => {
    const executeOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => true
    );
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
      oristudioCpViewport: {
        gridVisible: true,
        snapToGrid: false,
        snapToVertices: false,
        snapToLines: false,
      },
      executeOristudioCpCommand,
    });
    const canvas = setCanvasClientRect(container);

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Select lasso"]')?.click();
      await Promise.resolve();
    });

    act(() => {
      canvas.dispatchEvent(
        new MouseEvent('pointerdown', {
          bubbles: true,
          button: 0,
          clientX: 360,
          clientY: 348,
        })
      );
      canvas.dispatchEvent(
        new MouseEvent('pointermove', {
          bubbles: true,
          button: 0,
          clientX: 477.6,
          clientY: 348,
        })
      );
      canvas.dispatchEvent(
        new MouseEvent('pointermove', {
          bubbles: true,
          button: 0,
          clientX: 477.6,
          clientY: 230.4,
        })
      );
    });
    expect(container.querySelector('.cp-command-preview')).not.toBeNull();

    await act(async () => {
      canvas.dispatchEvent(
        new MouseEvent('pointerup', {
          bubbles: true,
          button: 0,
          clientX: 360,
          clientY: 230.4,
        })
      );
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenCalledOnce();
    const [operation, payload] = executeOristudioCpCommand.mock.calls[0] ?? [];
    expect(operation).toBe('SelectLasso');
    expect(payload?.points?.length).toBeGreaterThanOrEqual(3);
    expect(payload?.selection_distance).toBeGreaterThan(0);
  });
});
