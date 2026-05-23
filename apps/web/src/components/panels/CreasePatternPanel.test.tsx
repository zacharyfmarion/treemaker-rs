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
  OristudioCpCommandResult,
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
  const previewOristudioCpCommand = vi.fn(async () => ({
    segments: [],
    circles: [],
    points: [],
    diagnostics: [],
  }));
  useWorkspaceStore.setState(
    {
      ...useWorkspaceStore.getInitialState(),
      project,
      status,
      engineReady: true,
      optimizeScale,
      buildCreasePattern,
      previewOristudioCpCommand,
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
  return { container, buildCreasePattern, optimizeScale, previewOristudioCpCommand };
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

function camvDiagnosticResult(): OristudioCpCommandResult {
  return {
    operation: 'CheckCamv',
    status: 'OracleTested',
    diagnostics: ['Check CAMV found 1 issue(s)'],
    diagnostic_entries: [
      {
        id: 'CheckCamv-1',
        kind: 'CheckCamv',
        severity: 'error',
        message: 'Flat-foldability violation: Maekawa',
        point: { x: 0, y: 0 },
        rule: 'Maekawa',
      },
    ],
  };
}

function editableCpStateWithCircleSet(): OristudioCpDocumentState {
  const state = editableCpState();
  state.summary.circles = 4;
  state.document.crease_pattern.circles = [
    {
      x: 0,
      y: 0,
      r: 0.25,
      color: 'Cyan3',
      customized: 0,
      customized_color: { red: 100, green: 200, blue: 200 },
    },
    {
      x: 1,
      y: 0,
      r: 0.25,
      color: 'Cyan3',
      customized: 0,
      customized_color: { red: 100, green: 200, blue: 200 },
    },
    {
      x: 2,
      y: 0,
      r: 0.5,
      color: 'Cyan3',
      customized: 0,
      customized_color: { red: 100, green: 200, blue: 200 },
    },
    {
      x: 3,
      y: 0,
      r: 1,
      color: 'Cyan3',
      customized: 0,
      customized_color: { red: 100, green: 200, blue: 200 },
    },
  ];
  return state;
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

function setNumberInputValue(input: HTMLInputElement, value: string) {
  const valueSetter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
  valueSetter?.call(input, value);
  input.dispatchEvent(new Event('input', { bubbles: true }));
}

function setTextAreaValue(textarea: HTMLTextAreaElement, value: string) {
  const valueSetter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value')?.set;
  valueSetter?.call(textarea, value);
  textarea.dispatchEvent(new Event('input', { bubbles: true }));
}

function setSelectValue(select: HTMLSelectElement, value: string) {
  const valueSetter = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'value')?.set;
  valueSetter?.call(select, value);
  select.dispatchEvent(new Event('change', { bubbles: true }));
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

    expect(mvButton?.getAttribute('aria-pressed')).toBe('true');
    expect(rolesButton?.getAttribute('aria-pressed')).toBe('false');
    expect(container.querySelector('.crease--fold-mountain')).not.toBeNull();
    expect(container.querySelector('.crease--fold-valley')).not.toBeNull();
    expect(container.querySelector('.crease--kind-hinge')).toBeNull();

    act(() => {
      rolesButton?.click();
    });

    expect(useWorkspaceStore.getState().creaseColorMode).toBe('agrh');
    expect(container.querySelector('.crease--kind-hinge')).not.toBeNull();
    expect(container.querySelector('.crease--fold-valley')).toBeNull();
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
    expect(container.textContent).toContain('Select crease: Drag selection box');
    expect(container.textContent).toContain('Line M');
    expect(container.textContent).toContain('2 lines');
    expect(container.querySelector('button[aria-label="Mountain"]')?.textContent).toContain('M');
    expect(container.querySelector('button[aria-label="Valley"]')?.textContent).toContain('V');
    expect(container.querySelector('button[aria-label="Edge"]')?.textContent).toContain('E');
    expect(container.querySelector('button[aria-label="Auxiliary"]')?.textContent).toContain('A');

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
    expect(drawCreaseButton?.getAttribute('aria-disabled')).toBe('false');
    expect(drawCreaseButton?.getAttribute('data-ui-status')).toBe('ready');
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
    expect(container.querySelector('button[aria-label="Select crease"]')?.hasAttribute('data-active')).toBe(
      true
    );

    act(() => {
      drawCreaseButton?.click();
    });
    expect(container.textContent).toContain('Draw crease: Drag crease endpoint');
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
      useWorkspaceStore.getState().setOristudioCpSelection({
        ...useWorkspaceStore.getState().oristudioCpSelection,
        lines: [1],
      });
    });

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Make mountain"]')?.click();
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenCalledWith('CreaseMakeMountain', {
      line_ids: [1],
    });
  });

  it('routes active unselect-crease clicks through the command instead of direct selection', async () => {
    const executeOristudioCpCommand = vi.fn(async () => true);
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
      executeOristudioCpCommand,
    });

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Unselect crease"]')?.click();
      await Promise.resolve();
    });

    await act(async () => {
      container.querySelector<SVGLineElement>('[data-cp-line-hit-id="1"]')?.dispatchEvent(
        new MouseEvent('click', { bubbles: true })
      );
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenCalledWith(
      'CreaseUnselect',
      expect.objectContaining({
        line_ids: [1],
        selection_distance: expect.any(Number),
      })
    );
    expect(useWorkspaceStore.getState().oristudioCpSelection.lines).toEqual([]);
  });

  it('defaults editable CP interaction to repeatable drag-box crease selection', async () => {
    const executeOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => true
    );
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
      executeOristudioCpCommand,
    });
    const canvas = setCanvasClientRect(container);
    const selectButton = container.querySelector<HTMLButtonElement>(
      'button[aria-label="Select crease"]'
    );

    expect(selectButton?.hasAttribute('data-active')).toBe(true);

    act(() => {
      canvas.dispatchEvent(
        new MouseEvent('pointerdown', {
          bubbles: true,
          button: 0,
          clientX: 300,
          clientY: 300,
        })
      );
      canvas.dispatchEvent(
        new MouseEvent('pointermove', {
          bubbles: true,
          button: 0,
          clientX: 420,
          clientY: 420,
        })
      );
    });
    expect(container.querySelector('.cp-command-box-preview')).not.toBeNull();

    await act(async () => {
      canvas.dispatchEvent(
        new MouseEvent('pointerup', {
          bubbles: true,
          button: 0,
          clientX: 420,
          clientY: 420,
        })
      );
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenCalledOnce();
    const [operation, payload] = executeOristudioCpCommand.mock.calls[0] ?? [];
    expect(operation).toBe('CreaseSelect');
    expect(payload?.points).toHaveLength(2);
    expect(payload?.replace_selection).toBe(true);
    expect(selectButton?.hasAttribute('data-active')).toBe(true);
  });

  it('clears the selection on Escape without cancelling the default box-select mode', () => {
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
    });
    const selectButton = container.querySelector<HTMLButtonElement>(
      'button[aria-label="Select crease"]'
    );
    const body = container.querySelector<HTMLElement>('.cp-panel__body');

    act(() => {
      useWorkspaceStore.getState().setOristudioCpSelection({
        ...useWorkspaceStore.getState().oristudioCpSelection,
        lines: [1, 2],
      });
    });
    expect(container.textContent).toContain('2 selected');

    act(() => {
      body?.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true })
      );
    });

    expect(useWorkspaceStore.getState().oristudioCpSelection.lines).toEqual([]);
    expect(selectButton?.hasAttribute('data-active')).toBe(true);
    expect(container.textContent).toContain('Select crease: Drag selection box');
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
      useWorkspaceStore.getState().setOristudioCpSelection({
        ...useWorkspaceStore.getState().oristudioCpSelection,
        lines: [1],
      });
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

  it('runs flat-foldable boundary checks from a closed drag path', async () => {
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
        .querySelector<HTMLButtonElement>('button[aria-label="Flat-foldable boundary check"]')
        ?.click();
      await Promise.resolve();
    });
    expect(container.textContent).toContain(
      'Flat-foldable boundary check: Draw a closed boundary loop'
    );

    act(() => {
      canvas.dispatchEvent(
        new MouseEvent('pointerdown', {
          bubbles: true,
          button: 0,
          clientX: 300,
          clientY: 300,
        })
      );
      canvas.dispatchEvent(
        new MouseEvent('pointermove', {
          bubbles: true,
          button: 0,
          clientX: 420,
          clientY: 300,
        })
      );
      canvas.dispatchEvent(
        new MouseEvent('pointermove', {
          bubbles: true,
          button: 0,
          clientX: 360,
          clientY: 420,
        })
      );
    });

    await act(async () => {
      canvas.dispatchEvent(
        new MouseEvent('pointerup', {
          bubbles: true,
          button: 0,
          clientX: 300,
          clientY: 300,
        })
      );
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenCalledOnce();
    const [operation, payload] = executeOristudioCpCommand.mock.calls[0] ?? [];
    expect(operation).toBe('FlatFoldableCheck');
    expect(payload?.points?.length).toBeGreaterThanOrEqual(3);
    expect(payload?.selection_distance).toEqual(expect.any(Number));
  });

  it('shows contextual line-type controls before applying selected-type commands', async () => {
    const executeOristudioCpCommand = vi.fn(async () => true);
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
      executeOristudioCpCommand,
    });

    act(() => {
      useWorkspaceStore.getState().setOristudioCpSelection({
        ...useWorkspaceStore.getState().oristudioCpSelection,
        lines: [1],
      });
    });

    await act(async () => {
      container
        .querySelector<HTMLButtonElement>('button[aria-label="Replace selected line type"]')
        ?.click();
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).not.toHaveBeenCalled();
    const fromSelect = container.querySelector<HTMLSelectElement>(
      'select[aria-label="Replace from line type"]'
    );
    const toSelect = container.querySelector<HTMLSelectElement>(
      'select[aria-label="Replace to line type"]'
    );
    expect(fromSelect?.value).toBe('Any');
    expect(toSelect?.value).toBe('Edge');
    act(() => {
      if (fromSelect) setSelectValue(fromSelect, 'Valley');
      if (toSelect) setSelectValue(toSelect, 'Aux');
    });

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button.cp-context-panel__apply')?.click();
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenCalledWith('ReplaceLineTypeSelect', {
      line_ids: [1],
      custom_from_line_type: 'Valley',
      custom_to_line_type: 'Aux',
    });
  });

  it('passes contextual fix-inaccurate options only after apply', async () => {
    const executeOristudioCpCommand = vi.fn(async () => true);
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
      executeOristudioCpCommand,
    });

    act(() => {
      useWorkspaceStore.getState().setOristudioCpSelection({
        ...useWorkspaceStore.getState().oristudioCpSelection,
        lines: [1],
      });
    });

    await act(async () => {
      container
        .querySelector<HTMLButtonElement>('button[aria-label="Fix inaccurate creases"]')
        ?.click();
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).not.toHaveBeenCalled();
    const precisionInput = container.querySelector<HTMLInputElement>('input[aria-label="Fix precision"]');
    const bpCheckbox = container.querySelector<HTMLInputElement>('input[aria-label="Use BP fix targets"]');
    expect(precisionInput?.value).toBe('0.05');
    expect(bpCheckbox?.checked).toBe(true);
    act(() => {
      if (precisionInput) setNumberInputValue(precisionInput, '0.02');
      bpCheckbox?.click();
    });

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button.cp-context-panel__apply')?.click();
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenCalledWith('FixInaccurate', {
      line_ids: [1],
      fix_precision: 0.02,
      fix_precision_use_bp: false,
      fix_precision_use_22_5: true,
    });
  });

  it('runs diagnostic checks and renders latest diagnostic markers', async () => {
    const executeOristudioCpCommand = vi.fn(async () => true);
    const state = editableCpState();
    state.lastCommandResult = {
      operation: 'Check1',
      status: 'OracleTested',
      diagnostics: ['Check1 found 1 issue(s)'],
      diagnostic_entries: [
        {
          id: 'Check1-1',
          kind: 'Check1',
          severity: 'error',
          message: 'Overlapping or contained non-auxiliary creases',
          point: { x: 0, y: 0 },
          segments: state.document.crease_pattern.line_segments,
          rule: 'Check1',
        },
      ],
    };
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: state,
      executeOristudioCpCommand,
    });

    expect(container.querySelectorAll('.cp-diagnostic-segment')).toHaveLength(2);
    expect(container.querySelector('.cp-diagnostic-point')).not.toBeNull();

    act(() => {
      container
        .querySelector<SVGLineElement>('.cp-diagnostic-segment')
        ?.dispatchEvent(new MouseEvent('pointerdown', { bubbles: true, button: 0 }));
    });

    expect(useWorkspaceStore.getState().oristudioCpActiveDiagnosticId).toBe('Check1-1');
    expect(container.querySelector('.cp-diagnostic-segment--active')).not.toBeNull();
    expect(transformMocks.setTransform).toHaveBeenCalled();

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Check overlaps"]')?.click();
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenCalledWith('Check1', {
      line_ids: [],
    });
  });

  it('renders point-only CAMV diagnostics without segment arrays', () => {
    const state = editableCpState();
    state.lastCommandResult = {
      operation: 'DrawCreaseRestricted',
      status: 'OracleTested',
      diagnostics: ['Changed 1 line(s)'],
    };
    const camvResult = camvDiagnosticResult();

    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: state,
      oristudioCpCamvResult: camvResult,
    });

    expect(container.querySelectorAll('.cp-diagnostic-segment')).toHaveLength(0);
    expect(container.querySelector('.cp-diagnostic-point')).not.toBeNull();
    expect(container.querySelector('.cp-diagnostic-point')?.getAttribute('data-severity')).toBe(
      'error'
    );
    expect(container.querySelector('.cp-diagnostic-point__cross')).not.toBeNull();
    expect(container.querySelector('.cp-diagnostic-hud')?.textContent).toContain('1 CAMV Error');
    expect(container.querySelector('.cp-diagnostic-hud')?.textContent).toContain(
      'Flat-foldability violation: Maekawa'
    );
  });

  it('does not show the diagnostic HUD for ordinary edit command results', () => {
    const state = editableCpState();
    state.lastCommandResult = {
      operation: 'DrawCreaseRestricted',
      status: 'OracleTested',
      diagnostics: ['Changed 1 line(s)'],
    };

    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: state,
    });

    expect(container.querySelector('.cp-diagnostic-hud')).toBeNull();
  });

  it('does not show an always-on CAMV OK result as a floating HUD', () => {
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
      oristudioCpCamvResult: {
        operation: 'CheckCamv',
        status: 'OracleTested',
        diagnostics: ['Check CAMV passed'],
        diagnostic_entries: [],
      },
    });

    expect(container.querySelector('.cp-diagnostic-hud')).toBeNull();
  });

  it('passes contextual circle color options with selected circles only after apply', async () => {
    const executeOristudioCpCommand = vi.fn(async () => true);
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
      executeOristudioCpCommand,
    });

    act(() => {
      container.querySelector<SVGCircleElement>('.cp-circle')?.dispatchEvent(
        new MouseEvent('click', { bubbles: true })
      );
    });
    expect(useWorkspaceStore.getState().oristudioCpSelection.circles).toEqual([1]);

    await act(async () => {
      container
        .querySelector<HTMLButtonElement>('button[aria-label="Change circle color"]')
        ?.click();
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).not.toHaveBeenCalled();
    const redInput = container.querySelector<HTMLInputElement>(
      'input[aria-label="Circle color red"]'
    );
    const greenInput = container.querySelector<HTMLInputElement>(
      'input[aria-label="Circle color green"]'
    );
    const blueInput = container.querySelector<HTMLInputElement>(
      'input[aria-label="Circle color blue"]'
    );
    expect(redInput?.value).toBe('100');
    act(() => {
      if (redInput) setNumberInputValue(redInput, '10');
      if (greenInput) setNumberInputValue(greenInput, '20');
      if (blueInput) setNumberInputValue(blueInput, '30');
    });

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button.cp-context-panel__apply')?.click();
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenCalledWith('CircleChangeColor', {
      line_ids: [],
      circle_ids: [1],
      custom_circle_color: { red: 10, green: 20, blue: 30 },
    });
  });

  it('runs organize circles as an immediate annotation cleanup command', async () => {
    const executeOristudioCpCommand = vi.fn(async () => true);
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
      executeOristudioCpCommand,
    });

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Organize circles"]')?.click();
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenCalledWith('OrganizeCircles', {
      line_ids: [],
    });
  });

  it('shows active tool inputs for line division count and sends the edited count', async () => {
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
        .querySelector<HTMLButtonElement>('button[aria-label="Divide line by count"]')
        ?.click();
      await Promise.resolve();
    });

    expect(
      container.querySelector<HTMLElement>('section[aria-label="Crease pattern tool options"]')
    ).not.toBeNull();
    const countInput = container.querySelector<HTMLInputElement>('input[aria-label="Division count"]');
    expect(countInput?.value).toBe('2');
    act(() => {
      if (countInput) setNumberInputValue(countInput, '5');
    });
    expect(useWorkspaceStore.getState().oristudioCpHistoryPast).toHaveLength(0);
    expect(useWorkspaceStore.getState().historyPast).toHaveLength(0);

    await act(async () => {
      canvas.dispatchEvent(
        new MouseEvent('pointerdown', {
          bubbles: true,
          button: 0,
          clientX: 360,
          clientY: 348,
        })
      );
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenCalledOnce();
    const [operation, payload] = executeOristudioCpCommand.mock.calls[0] ?? [];
    expect(operation).toBe('LineSegmentDivision');
    expect(payload?.division_count).toBe(5);
  });

  it('shows active tool inputs for line division ratio and sends the edited ratio', async () => {
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
        .querySelector<HTMLButtonElement>('button[aria-label="Divide line by ratio"]')
        ?.click();
      await Promise.resolve();
    });

    const leftInput = container.querySelector<HTMLInputElement>(
      'input[aria-label="Left segment ratio"]'
    );
    const rightInput = container.querySelector<HTMLInputElement>(
      'input[aria-label="Right segment ratio"]'
    );
    expect(leftInput?.value).toBe('1');
    expect(rightInput?.value).toBe('sqrt(2)');
    expect(container.textContent).toContain('Computed ratio 1 : 1.414');

    act(() => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Use 1:2 ratio"]')?.click();
    });
    expect(leftInput?.value).toBe('1');
    expect(rightInput?.value).toBe('2');

    act(() => {
      if (leftInput) setNumberInputValue(leftInput, '2');
      if (rightInput) setNumberInputValue(rightInput, '3');
    });

    await act(async () => {
      canvas.dispatchEvent(
        new MouseEvent('pointerdown', {
          bubbles: true,
          button: 0,
          clientX: 360,
          clientY: 348,
        })
      );
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenCalledOnce();
    const [operation, payload] = executeOristudioCpCommand.mock.calls[0] ?? [];
    expect(operation).toBe('LineSegmentRatioSet');
    expect(payload?.ratio_s).toBe(2);
    expect(payload?.ratio_t).toBe(3);
  });

  it('records length measurements locally without mutating CP history', async () => {
    const executeOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => true
    );
    const { container, previewOristudioCpCommand } = renderPanel(
      createSampleProject(),
      'crease_pattern_ready',
      {
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
      }
    );
    const canvas = setCanvasClientRect(container);

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Measure length 1"]')?.click();
      await Promise.resolve();
    });

    expect(container.textContent).toContain('Measure length 1: Pick first point');
    expect(
      container.querySelector<HTMLElement>('[data-measurement-slot="length1"]')?.textContent
    ).toBe('L1-');

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
    expect(container.textContent).toContain('Measure length 1: Pick second point');

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

    expect(
      container.querySelector<HTMLElement>('[data-measurement-slot="length1"]')?.textContent
    ).toBe('L180');
    expect(container.textContent).toContain('Measure length 1: Pick first point');
    expect(executeOristudioCpCommand).not.toHaveBeenCalled();
    expect(previewOristudioCpCommand).not.toHaveBeenCalled();
    expect(useWorkspaceStore.getState().oristudioCpHistoryPast).toHaveLength(0);
    expect(useWorkspaceStore.getState().historyPast).toHaveLength(0);
  });

  it('records oriented angle measurements locally without mutating CP history', async () => {
    const executeOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => true
    );
    const { container, previewOristudioCpCommand } = renderPanel(
      createSampleProject(),
      'crease_pattern_ready',
      {
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
      }
    );
    const canvas = setCanvasClientRect(container);

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Measure angle 1"]')?.click();
      await Promise.resolve();
    });

    for (const [clientX, clientY] of [
      [477.6, 348],
      [360, 348],
      [360, 230.4],
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

    expect(
      container.querySelector<HTMLElement>('[data-measurement-slot="angle1"]')?.textContent
    ).toBe('A190 deg');
    expect(executeOristudioCpCommand).not.toHaveBeenCalled();
    expect(previewOristudioCpCommand).not.toHaveBeenCalled();
    expect(useWorkspaceStore.getState().oristudioCpHistoryPast).toHaveLength(0);
    expect(useWorkspaceStore.getState().historyPast).toHaveLength(0);
  });

  it('runs ready circle commands with circle previews and resolved model points', async () => {
    const executeOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => true
    );
    const previewOristudioCpCommand = vi.fn(async () => ({
      segments: [],
      circles: [
        {
          x: 0,
          y: 0,
          r: 80,
          color: 'Cyan3',
          customized: 0,
          customized_color: { red: 100, green: 200, blue: 200 },
        },
      ],
      points: [],
      diagnostics: [],
    }));
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
      previewOristudioCpCommand,
    });
    const canvas = setCanvasClientRect(container);

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Draw circle"]')?.click();
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
    });

    await act(async () => {
      canvas.dispatchEvent(
        new MouseEvent('pointermove', {
          bubbles: true,
          button: 0,
          clientX: 477.6,
          clientY: 348,
        })
      );
      await Promise.resolve();
    });

    expect(previewOristudioCpCommand).toHaveBeenCalledWith(
      'CircleDraw',
      expect.objectContaining({
        points: [
          { x: 0, y: 0 },
          expect.objectContaining({ x: expect.any(Number), y: expect.any(Number) }),
        ],
      })
    );
    expect(container.querySelector('circle.cp-command-preview')).not.toBeNull();

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
    expect(operation).toBe('CircleDraw');
    expect(payload?.points?.[0]).toEqual({ x: 0, y: 0 });
    expect(payload?.points?.[1].x).toBeCloseTo(80);
    expect(payload?.points?.[1].y).toBeCloseTo(0);
  });

  it('applies selected-circle circle modes from the contextual panel', async () => {
    const executeOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => true
    );
    const previewOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => ({
        segments: [
          {
            a: { x: 0, y: 0 },
            b: { x: 1, y: 1 },
            color: 'Purple8',
            active: 'Inactive0',
            selected: 0,
            customized: 0,
            customized_color: { red: 0, green: 0, blue: 0 },
          },
        ],
        circles: [
          {
            x: 3,
            y: 0,
            r: 2,
            color: 'Magenta5',
            customized: 0,
            customized_color: { red: 100, green: 200, blue: 200 },
          },
        ],
        points: [],
        diagnostics: [],
      })
    );
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpStateWithCircleSet(),
      executeOristudioCpCommand,
      previewOristudioCpCommand,
    });

    const circles = () => Array.from(container.querySelectorAll<SVGCircleElement>('.cp-circle'));

    act(() => {
      circles()[0]?.dispatchEvent(new MouseEvent('click', { bubbles: true }));
      circles()[1]?.dispatchEvent(new MouseEvent('click', { bubbles: true, shiftKey: true }));
    });

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Circle tangent line"]')?.click();
      await Promise.resolve();
    });

    expect(container.textContent).toContain('Circle selection');
    expect(previewOristudioCpCommand).toHaveBeenCalledWith(
      'CircleDrawTangentLine',
      expect.objectContaining({
        circle_ids: [1, 2],
        line_color: 'Red1',
      })
    );
    const applyCircle = Array.from(container.querySelectorAll<HTMLButtonElement>('button')).find(
      (button) => button.textContent?.trim() === 'Apply circle'
    );
    expect(applyCircle?.disabled).toBe(false);

    await act(async () => {
      applyCircle?.click();
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenLastCalledWith('CircleDrawTangentLine', {
      line_ids: [],
      circle_ids: [1, 2],
      line_color: 'Red1',
    });

    previewOristudioCpCommand.mockClear();
    act(() => {
      useWorkspaceStore.getState().setOristudioCpSelection({
        lines: [],
        vertices: [],
        points: [],
        circles: [1, 3, 4],
        texts: [],
        faces: [],
      });
    });

    await act(async () => {
      container
        .querySelector<HTMLButtonElement>('button[aria-label="Concentric from selection"]')
        ?.click();
      await Promise.resolve();
    });

    expect(previewOristudioCpCommand).toHaveBeenCalledWith(
      'CircleDrawConcentricSelect',
      expect.objectContaining({
        circle_ids: [1, 3, 4],
      })
    );

    await act(async () => {
      Array.from(container.querySelectorAll<HTMLButtonElement>('button'))
        .find((button) => button.textContent?.trim() === 'Apply circle')
        ?.click();
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenLastCalledWith('CircleDrawConcentricSelect', {
      line_ids: [],
      circle_ids: [1, 3, 4],
    });
  });

  it('creates a tangent line from one selected circle plus a clicked point', async () => {
    const executeOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => true
    );
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpStateWithCircleSet(),
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
      container
        .querySelector<SVGCircleElement>('.cp-circle')
        ?.dispatchEvent(new MouseEvent('click', { bubbles: true }));
    });

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Circle tangent line"]')?.click();
      await Promise.resolve();
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
    expect(operation).toBe('CircleDrawTangentLine');
    expect(payload?.circle_ids).toEqual([1]);
    expect(payload?.line_color).toBe('Red1');
    expect(payload?.points).toHaveLength(1);
  });

  it('runs regular polygon with contextual corner count and active line color', async () => {
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
      container.querySelector<HTMLButtonElement>('button[aria-label="Valley"]')?.click();
      container.querySelector<HTMLButtonElement>('button[aria-label="Regular polygon"]')?.click();
      await Promise.resolve();
    });

    const cornersInput = container.querySelector<HTMLInputElement>(
      'input[aria-label="Polygon corners"]'
    );
    expect(cornersInput?.value).toBe('5');
    act(() => {
      if (cornersInput) setNumberInputValue(cornersInput, '4');
    });

    for (const [clientX, clientY] of [
      [360, 348],
      [477.6, 348],
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
    expect(operation).toBe('PolygonSetNoCorners');
    expect(payload?.polygon_corners).toBe(4);
    expect(payload?.line_color).toBe('Blue2');
    expect(payload?.points).toHaveLength(2);
  });

  it('runs default base generators with the active line color', async () => {
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
      container.querySelector<HTMLButtonElement>('button[aria-label="Valley"]')?.click();
      container.querySelector<HTMLButtonElement>('button[aria-label="Blintz base"]')?.click();
      await Promise.resolve();
    });

    for (const [clientX, clientY] of [
      [66, 54],
      [654, 642],
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
    expect(operation).toBe('DrawBlintz');
    expect(payload?.line_color).toBe('Blue2');
    expect(payload?.points?.[0]).toEqual({ x: -200, y: 200 });
    expect(payload?.points?.[1]).toEqual({ x: 200, y: -200 });
  });

  it('collects Voronoi seed presses, previews them, and applies from the context panel', async () => {
    const executeOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => true
    );
    const previewOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => ({
        segments: [
          {
            a: { x: 0, y: 0 },
            b: { x: 1, y: 1 },
            color: 'Magenta5',
            active: 'Inactive0',
            selected: 0,
            customized: 0,
            customized_color: { red: 0, green: 0, blue: 0 },
          },
        ],
        circles: [],
        points: [
          { x: 0, y: 0 },
          { x: 1, y: 0 },
        ],
        diagnostics: [],
      })
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
      previewOristudioCpCommand,
    });
    const canvas = setCanvasClientRect(container);

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Valley"]')?.click();
      container.querySelector<HTMLButtonElement>('button[aria-label="Voronoi"]')?.click();
      await Promise.resolve();
    });

    expect(container.textContent).toContain('Voronoi seeds');
    expect(container.textContent).toContain('0 seed presses pending');
    const emptyApply = Array.from(container.querySelectorAll<HTMLButtonElement>('button')).find(
      (button) => button.textContent?.trim() === 'Apply Voronoi'
    );
    expect(emptyApply?.disabled).toBe(true);

    for (const [clientX, clientY] of [
      [360, 348],
      [477.6, 348],
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

    expect(previewOristudioCpCommand).toHaveBeenCalled();
    const lastPreviewCall =
      previewOristudioCpCommand.mock.calls[previewOristudioCpCommand.mock.calls.length - 1];
    const previewPayload = lastPreviewCall?.[1] as OristudioCpCommandPayload | undefined;
    expect(lastPreviewCall?.[0]).toBe('VoronoiCreate');
    expect(previewPayload?.line_color).toBe('Blue2');
    expect(previewPayload?.selection_distance).toBeGreaterThan(0);
    expect(previewPayload?.points).toHaveLength(2);
    expect(container.querySelector('.cp-command-candidate')).not.toBeNull();
    expect(container.querySelectorAll('.cp-command-candidate-point')).toHaveLength(2);
    expect(container.textContent).toContain('2 seed presses pending');

    const applyButton = Array.from(container.querySelectorAll<HTMLButtonElement>('button')).find(
      (button) => button.textContent?.trim() === 'Apply Voronoi'
    );
    await act(async () => {
      applyButton?.click();
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenCalledOnce();
    const [operation, payload] = executeOristudioCpCommand.mock.calls[0] ?? [];
    expect(operation).toBe('VoronoiCreate');
    expect(payload?.line_color).toBe('Blue2');
    expect(payload?.points).toHaveLength(2);
    expect(container.textContent).toContain('0 seed presses pending');
  });

  it('clears pending Voronoi seeds without running a command', async () => {
    const executeOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => true
    );
    const previewOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => ({
        segments: [],
        circles: [],
        points: [{ x: 0, y: 0 }],
        diagnostics: [],
      })
    );
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
      executeOristudioCpCommand,
      previewOristudioCpCommand,
    });
    const canvas = setCanvasClientRect(container);

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Voronoi"]')?.click();
      await Promise.resolve();
    });

    await act(async () => {
      canvas.dispatchEvent(
        new MouseEvent('pointerdown', {
          bubbles: true,
          button: 0,
          clientX: 360,
          clientY: 348,
        })
      );
      await Promise.resolve();
    });

    expect(container.textContent).toContain('1 seed press pending');
    const clearButton = Array.from(container.querySelectorAll<HTMLButtonElement>('button')).find(
      (button) => button.textContent?.trim() === 'Clear seeds'
    );
    await act(async () => {
      clearButton?.click();
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).not.toHaveBeenCalled();
    expect(container.textContent).toContain('0 seed presses pending');
  });

  it('creates text annotations with the contextual text draft', async () => {
    const executeOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => true
    );
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
      executeOristudioCpCommand,
    });
    const canvas = setCanvasClientRect(container);

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Text annotation"]')?.click();
      await Promise.resolve();
    });
    const textArea = container.querySelector<HTMLTextAreaElement>(
      'textarea[aria-label="Text annotation content"]'
    );
    expect(textArea).not.toBeNull();

    await act(async () => {
      if (textArea) setTextAreaValue(textArea, 'new label');
      canvas.dispatchEvent(
        new MouseEvent('pointerdown', {
          bubbles: true,
          button: 0,
          clientX: 360,
          clientY: 348,
        })
      );
      await Promise.resolve();
    });

    expect(executeOristudioCpCommand).toHaveBeenCalledOnce();
    const [operation, payload] = executeOristudioCpCommand.mock.calls[0] ?? [];
    expect(operation).toBe('Text');
    expect(payload).toMatchObject({
      text_action: 'Create',
      text_content: 'new label',
      points: [{ x: 0, y: 0 }],
    });
  });

  it('edits, deletes, and drags selected text annotations', async () => {
    const executeOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => true
    );
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
      executeOristudioCpCommand,
    });
    const canvas = setCanvasClientRect(container);
    const textElement = container.querySelector<SVGTextElement>('[data-cp-text-id="1"]');
    expect(textElement).not.toBeNull();

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Text annotation"]')?.click();
      await Promise.resolve();
    });

    await act(async () => {
      textElement?.dispatchEvent(
        new MouseEvent('pointerdown', {
          bubbles: true,
          button: 0,
          clientX: 418.8,
          clientY: 254.4,
        })
      );
      await Promise.resolve();
    });
    expect(useWorkspaceStore.getState().oristudioCpSelection.texts).toEqual([1]);

    const textArea = container.querySelector<HTMLTextAreaElement>(
      'textarea[aria-label="Text annotation content"]'
    );
    await act(async () => {
      if (textArea) setTextAreaValue(textArea, 'updated note');
      Array.from(container.querySelectorAll<HTMLButtonElement>('button'))
        .find((button) => button.textContent?.trim() === 'Apply text')
        ?.click();
      await Promise.resolve();
    });
    expect(executeOristudioCpCommand).toHaveBeenCalledWith(
      'Text',
      expect.objectContaining({
        text_action: 'SetContent',
        text_ids: [1],
        text_content: 'updated note',
      })
    );

    await act(async () => {
      textElement?.dispatchEvent(
        new MouseEvent('pointerdown', {
          bubbles: true,
          button: 0,
          clientX: 418.8,
          clientY: 254.4,
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
      canvas.dispatchEvent(
        new MouseEvent('pointerup', {
          bubbles: true,
          button: 0,
          clientX: 477.6,
          clientY: 230.4,
        })
      );
      await Promise.resolve();
    });
    expect(executeOristudioCpCommand).toHaveBeenCalledWith(
      'Text',
      expect.objectContaining({
        text_action: 'Move',
        text_ids: [1],
        points: expect.any(Array),
      })
    );

    await act(async () => {
      Array.from(container.querySelectorAll<HTMLButtonElement>('button'))
        .find((button) => button.textContent?.trim() === 'Delete text')
        ?.click();
      await Promise.resolve();
    });
    expect(executeOristudioCpCommand).toHaveBeenCalledWith(
      'Text',
      expect.objectContaining({
        text_action: 'DeleteSelected',
        text_ids: [1],
      })
    );
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

  it('runs draw crease as a line-type-aware drag action with synchronous preview', async () => {
    const executeOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => true
    );
    const previewOristudioCpCommand = vi.fn(async () => ({
      segments: [
        {
          a: { x: 0, y: 0 },
          b: { x: 80, y: 0 },
          color: 'Red1',
          active: 'Inactive0',
          selected: 0,
          customized: 0,
          customized_color: { red: 0, green: 0, blue: 0 },
        },
      ],
      circles: [],
      points: [{ x: 40, y: 0 }],
      diagnostics: [],
    }));
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
      previewOristudioCpCommand,
    });
    const canvas = setCanvasClientRect(container);

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Valley"]')?.click();
      container.querySelector<HTMLButtonElement>('button[aria-label="Draw crease"]')?.click();
      await Promise.resolve();
    });
    expect(container.textContent).toContain('Line V');

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
    });

    expect(previewOristudioCpCommand).not.toHaveBeenCalled();
    expect(container.querySelector('.cp-command-candidate')).not.toBeNull();

    await act(async () => {
      canvas.dispatchEvent(
        new MouseEvent('pointerup', {
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
    expect(operation).toBe('DrawCreaseFree');
    expect(payload?.line_color).toBe('Blue2');
    expect(payload?.selection_distance).toBeGreaterThan(0);
    expect(payload?.points).toHaveLength(2);
    expect(container.textContent).toContain('Draw crease: Drag crease endpoint');
  });

  it('snaps draw crease drag endpoints to existing Oriedita vertices', async () => {
    const executeOristudioCpCommand = vi.fn(
      async (_operationId: string, _payload?: OristudioCpCommandPayload) => true
    );
    const { container } = renderPanel(createSampleProject(), 'crease_pattern_ready', {
      documentMode: 'crease-pattern',
      importedCreasePattern: importedCpDocument(),
      oristudioCpDocument: editableCpState(),
      oristudioCpViewport: {
        gridVisible: false,
        snapToGrid: false,
        snapToVertices: true,
        snapToLines: true,
      },
      executeOristudioCpCommand,
    });
    const canvas = setCanvasClientRect(container);

    await act(async () => {
      container.querySelector<HTMLButtonElement>('button[aria-label="Draw crease"]')?.click();
      await Promise.resolve();
    });

    await act(async () => {
      canvas.dispatchEvent(
        new MouseEvent('pointerdown', {
          bubbles: true,
          button: 0,
          clientX: 360.2,
          clientY: 348.2,
        })
      );
      canvas.dispatchEvent(
        new MouseEvent('pointermove', {
          bubbles: true,
          button: 0,
          clientX: 361.7,
          clientY: 348.1,
        })
      );
      canvas.dispatchEvent(
        new MouseEvent('pointerup', {
          bubbles: true,
          button: 0,
          clientX: 361.7,
          clientY: 348.1,
        })
      );
      await Promise.resolve();
    });

    const [, payload] = executeOristudioCpCommand.mock.calls[0] ?? [];
    expect(payload?.points?.[0]).toEqual({ x: 0, y: 0 });
    expect(payload?.points?.[1]).toEqual({ x: 1, y: 0 });
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
