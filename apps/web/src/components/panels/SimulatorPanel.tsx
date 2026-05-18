import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
  type WheelEvent as ReactWheelEvent,
} from 'react';
import {
  Eye,
  EyeOff,
  Layers3,
  Pause,
  Play,
  RefreshCw,
  RotateCcw,
  Square,
  StepForward,
  Waves,
} from 'lucide-react';
import {
  createOrigamiSimulator,
  prepareFoldModel,
  type FoldDocument as SimulatorFoldDocument,
  type OrigamiSimulatorController,
  type PreparedOrigamiModel,
  type SimulationFrame,
} from '@treemaker/origami-simulator';
import {
  nextSimulatorOrbitView,
  type SimulatorOrbitView as SimulatorView,
} from '../../lib/simulatorOrbit';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { Button } from '../ui/Button';
import { IconButton } from '../ui/IconButton';
import { SegmentedControl } from '../ui/SegmentedControl';

type LoadState = 'idle' | 'loading' | 'ready' | 'empty' | 'error';
type SimulatorRenderMode = 'paper' | 'xray';

interface SimulatorViewSettings {
  renderMode: SimulatorRenderMode;
  showFaces: boolean;
  showEdges: boolean;
  showHiddenLines: boolean;
}

interface DragState {
  pointerId: number;
  x: number;
  y: number;
  yaw: number;
  pitch: number;
}

interface ProjectedPoint {
  x: number;
  y: number;
  depth: number;
}

const INITIAL_SETTLE_STEPS = 300;
const FOLD_CHANGE_IMMEDIATE_STEPS = 200;
const FOLD_CHANGE_SETTLE_BATCH = 200;
const FOLD_CHANGE_SETTLE_FRAMES = 40;
const FOLD_PLAY_STEP_BATCH = 160;
const FOLD_PLAY_PERCENT_PER_SECOND = 28;
const FOLD_STEP_PERCENT = 5;
const SETTLE_DELTA_EPSILON = 0.0002;
const INITIAL_FOLD_PERCENT = 0;
const DEFAULT_VIEW: SimulatorView = { yaw: 0, pitch: 0.38, zoom: 1 };
const DEFAULT_VIEW_SETTINGS: SimulatorViewSettings = {
  renderMode: 'paper',
  showFaces: true,
  showEdges: true,
  showHiddenLines: false,
};

export function SimulatorPanel() {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const controllerRef = useRef<OrigamiSimulatorController | null>(null);
  const modelRef = useRef<PreparedOrigamiModel | null>(null);
  const frameRef = useRef<SimulationFrame | null>(null);
  const rafRef = useRef<number | null>(null);
  const settleRafRef = useRef<number | null>(null);
  const viewRef = useRef<SimulatorView>({ ...DEFAULT_VIEW });
  const dragRef = useRef<DragState | null>(null);
  const foldPercentRef = useRef(INITIAL_FOLD_PERCENT);

  const creaseCount = useWorkspaceStore((state) => state.project.creases.length);
  const foldArtifacts = useWorkspaceStore((state) => state.foldArtifacts);
  const foldArtifactError = useWorkspaceStore((state) => state.foldArtifactError);
  const refreshFoldArtifacts = useWorkspaceStore((state) => state.refreshFoldArtifacts);
  const buildCreasePattern = useWorkspaceStore((state) => state.buildCreasePattern);

  const [foldPercent, setFoldPercent] = useState(INITIAL_FOLD_PERCENT);
  const [playing, setPlaying] = useState(false);
  const [loadState, setLoadState] = useState<LoadState>('idle');
  const [modelError, setModelError] = useState<string | null>(null);
  const [step, setStep] = useState(0);
  const [strain, setStrain] = useState(0);
  const [modelStats, setModelStats] = useState({ vertices: 0, triangles: 0 });
  const [viewSettings, setViewSettings] = useState<SimulatorViewSettings>(DEFAULT_VIEW_SETTINGS);

  const drawCurrentFrame = useCallback(() => {
    const canvas = canvasRef.current;
    const model = modelRef.current;
    const frame = frameRef.current;
    if (!canvas || !model || !frame) return;
    drawFrame(canvas, model, frame, viewRef.current, viewSettings);
    setStep(frame.step);
    setStrain(frame.diagnostics.maxEdgeStrain ?? 0);
  }, [viewSettings]);

  const stepSimulation = useCallback(
    (steps?: number): SimulationFrame | null => {
      const controller = controllerRef.current;
      if (!controller) return null;
      frameRef.current = controller.step(steps);
      drawCurrentFrame();
      return frameRef.current;
    },
    [drawCurrentFrame]
  );

  const applyFoldPercent = useCallback((percent: number) => {
    const next = clamp(percent, 0, 100);
    foldPercentRef.current = next;
    setFoldPercent(next);
    controllerRef.current?.setFoldPercent(next);
  }, []);

  const clearPlayback = useCallback(() => {
    if (rafRef.current !== null && typeof window !== 'undefined') {
      window.cancelAnimationFrame(rafRef.current);
    }
    rafRef.current = null;
  }, []);

  const clearSettling = useCallback(() => {
    if (settleRafRef.current !== null && typeof window !== 'undefined') {
      window.cancelAnimationFrame(settleRafRef.current);
    }
    settleRafRef.current = null;
  }, []);

  const startSettling = useCallback(
    (frames = FOLD_CHANGE_SETTLE_FRAMES) => {
      if (typeof window === 'undefined') return;
      clearSettling();
      let remaining = frames;
      let quietFrames = 0;

      const tick = () => {
        const previous = frameRef.current?.positions;
        const next = stepSimulation(FOLD_CHANGE_SETTLE_BATCH);
        if (!next) {
          settleRafRef.current = null;
          return;
        }

        const delta = previous ? maxPositionDelta(previous, next.positions) : Infinity;
        quietFrames = delta < SETTLE_DELTA_EPSILON ? quietFrames + 1 : 0;
        remaining -= 1;

        if (remaining > 0 && quietFrames < 3) {
          settleRafRef.current = window.requestAnimationFrame(tick);
        } else {
          settleRafRef.current = null;
        }
      };

      settleRafRef.current = window.requestAnimationFrame(tick);
    },
    [clearSettling, stepSimulation]
  );

  useEffect(() => {
    if (creaseCount === 0) {
      clearPlayback();
      clearSettling();
      setPlaying(false);
      setModelError(null);
      setLoadState('empty');
      return;
    }
    if (foldArtifacts) {
      setModelError(null);
      setLoadState('ready');
      return;
    }

    let cancelled = false;
    setModelError(null);
    setLoadState('loading');
    void refreshFoldArtifacts().then((artifacts) => {
      if (cancelled) return;
      setLoadState(artifacts ? 'ready' : 'error');
    });
    return () => {
      cancelled = true;
    };
  }, [clearPlayback, clearSettling, creaseCount, foldArtifacts, refreshFoldArtifacts]);

  useEffect(() => {
    clearPlayback();
    clearSettling();
    controllerRef.current?.dispose();
    controllerRef.current = null;
    modelRef.current = null;
    frameRef.current = null;
    setModelError(null);
    setModelStats({ vertices: 0, triangles: 0 });

    if (!foldArtifacts) return;

    try {
      const model = prepareFoldModel(
        (foldArtifacts.simulation_model?.fold ?? foldArtifacts.fold) as SimulatorFoldDocument,
        { triangulate: false }
      );
      const controller = createOrigamiSimulator({
        model,
        options: { foldPercent: foldPercentRef.current },
      });
      modelRef.current = model;
      controllerRef.current = controller;
      setModelStats({ vertices: model.vertexCount, triangles: model.faceCount });
      frameRef.current = controller.step(INITIAL_SETTLE_STEPS);
      setLoadState('ready');
      drawCurrentFrame();
      if (foldPercentRef.current !== 0) startSettling();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      console.warn('Failed to prepare simulator model', error);
      setPlaying(false);
      setModelError(message);
      setModelStats({ vertices: 0, triangles: 0 });
      setLoadState('error');
    }

    return () => {
      clearPlayback();
      clearSettling();
      controllerRef.current?.dispose();
      controllerRef.current = null;
      modelRef.current = null;
      frameRef.current = null;
      setModelStats({ vertices: 0, triangles: 0 });
    };
  }, [clearPlayback, clearSettling, drawCurrentFrame, foldArtifacts, startSettling]);

  const setFoldTarget = useCallback(
    (percent: number) => {
      clearPlayback();
      setPlaying(false);
      applyFoldPercent(percent);
      if (stepSimulation(FOLD_CHANGE_IMMEDIATE_STEPS)) {
        startSettling();
      }
    },
    [applyFoldPercent, clearPlayback, startSettling, stepSimulation]
  );

  const stepFoldTarget = useCallback(() => {
    setFoldTarget(
      Math.min(100, Math.floor(foldPercentRef.current / FOLD_STEP_PERCENT + 1) * FOLD_STEP_PERCENT)
    );
  }, [setFoldTarget]);

  const replayFromFlat = useCallback(() => {
    clearPlayback();
    clearSettling();
    setPlaying(false);
    controllerRef.current?.reset();
    applyFoldPercent(0);
    frameRef.current = controllerRef.current?.step(INITIAL_SETTLE_STEPS) ?? null;
    drawCurrentFrame();
  }, [applyFoldPercent, clearPlayback, clearSettling, drawCurrentFrame]);

  useEffect(() => {
    if (!playing || typeof window === 'undefined') return;
    clearSettling();

    if (foldPercentRef.current >= 100) {
      controllerRef.current?.reset();
      applyFoldPercent(0);
      frameRef.current = controllerRef.current?.step(INITIAL_SETTLE_STEPS) ?? null;
      drawCurrentFrame();
    }

    let previousTime: number | null = null;
    const tick = (time: number) => {
      if (previousTime === null) previousTime = time;
      const elapsedSeconds = Math.min(0.08, (time - previousTime) / 1000);
      previousTime = time;
      const nextPercent = Math.min(
        100,
        foldPercentRef.current + elapsedSeconds * FOLD_PLAY_PERCENT_PER_SECOND
      );

      applyFoldPercent(nextPercent);
      stepSimulation(FOLD_PLAY_STEP_BATCH);

      if (nextPercent >= 100) {
        rafRef.current = null;
        setPlaying(false);
        startSettling();
        return;
      }

      rafRef.current = window.requestAnimationFrame(tick);
    };
    rafRef.current = window.requestAnimationFrame(tick);
    return clearPlayback;
  }, [
    applyFoldPercent,
    clearPlayback,
    clearSettling,
    drawCurrentFrame,
    playing,
    startSettling,
    stepSimulation,
  ]);

  useEffect(() => {
    if (typeof ResizeObserver === 'undefined') return;
    const canvas = canvasRef.current;
    if (!canvas) return;
    const observer = new ResizeObserver(drawCurrentFrame);
    observer.observe(canvas);
    return () => observer.disconnect();
  }, [drawCurrentFrame]);

  useEffect(() => {
    drawCurrentFrame();
  }, [drawCurrentFrame]);

  const resetView = useCallback(() => {
    viewRef.current = { ...DEFAULT_VIEW };
    drawCurrentFrame();
  }, [drawCurrentFrame]);

  const handleCanvasPointerDown = (event: ReactPointerEvent<HTMLCanvasElement>) => {
    if (loadState !== 'ready') return;
    event.currentTarget.setPointerCapture(event.pointerId);
    dragRef.current = {
      pointerId: event.pointerId,
      x: event.clientX,
      y: event.clientY,
      yaw: viewRef.current.yaw,
      pitch: viewRef.current.pitch,
    };
  };

  const handleCanvasPointerMove = (event: ReactPointerEvent<HTMLCanvasElement>) => {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== event.pointerId) return;
    viewRef.current = nextSimulatorOrbitView(viewRef.current, drag, {
      x: event.clientX,
      y: event.clientY,
    });
    drawCurrentFrame();
  };

  const handleCanvasPointerEnd = (event: ReactPointerEvent<HTMLCanvasElement>) => {
    if (dragRef.current?.pointerId !== event.pointerId) return;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    dragRef.current = null;
  };

  const handleCanvasWheel = (event: ReactWheelEvent<HTMLCanvasElement>) => {
    if (loadState !== 'ready') return;
    event.preventDefault();
    viewRef.current = {
      ...viewRef.current,
      zoom: clamp(viewRef.current.zoom * Math.exp(-event.deltaY * 0.001), 0.45, 4),
    };
    drawCurrentFrame();
  };

  const errorDetail = modelError ?? foldArtifactError ?? 'Simulator unavailable';
  const statusLabel =
    loadState === 'ready'
      ? `${modelStats.vertices} vertices | ${modelStats.triangles} triangles`
      : loadState === 'loading'
        ? 'Loading'
        : loadState === 'empty'
          ? 'No crease pattern'
          : loadState === 'error'
            ? shortStatus(errorDetail)
            : 'Idle';

  return (
    <section className="panel-shell simulator-panel">
      <div className="panel-toolbar">
        <div className="panel-toolbar__group">
          <Waves size={14} />
          <span className="panel-title">Simulator</span>
        </div>
        <div className="panel-toolbar__group simulator-view-settings" aria-label="Simulator view settings">
          <SegmentedControl
            aria-label="Simulator render mode"
            value={viewSettings.renderMode}
            onChange={(renderMode) => setViewSettings((current) => ({ ...current, renderMode }))}
            options={[
              { value: 'paper', label: 'Paper', title: 'Paper rendering' },
              { value: 'xray', label: 'X-ray', title: 'X-ray rendering' },
            ]}
          />
          <IconButton
            size="sm"
            variant="toolbar"
            title="Faces"
            tooltipSide="bottom"
            isActive={viewSettings.showFaces}
            onClick={() => setViewSettings((current) => ({ ...current, showFaces: !current.showFaces }))}
          >
            <Square size={14} />
          </IconButton>
          <IconButton
            size="sm"
            variant="toolbar"
            title="Crease Lines"
            tooltipSide="bottom"
            isActive={viewSettings.showEdges}
            onClick={() => setViewSettings((current) => ({ ...current, showEdges: !current.showEdges }))}
          >
            {viewSettings.showEdges ? <Eye size={14} /> : <EyeOff size={14} />}
          </IconButton>
          <IconButton
            size="sm"
            variant="toolbar"
            title="Hidden Lines"
            tooltipSide="bottom"
            isActive={viewSettings.showHiddenLines}
            onClick={() =>
              setViewSettings((current) => ({
                ...current,
                showHiddenLines: !current.showHiddenLines,
              }))
            }
            disabled={!viewSettings.showEdges}
          >
            <Layers3 size={14} />
          </IconButton>
        </div>
      </div>
      <div className="panel-body simulator-panel__body">
        <canvas
          ref={canvasRef}
          className="simulator-canvas"
          aria-label="Origami folded-base simulator. Drag to rotate, scroll to zoom, double-click to reset view."
          title="Drag to rotate, scroll to zoom, double-click to reset view"
          onPointerDown={handleCanvasPointerDown}
          onPointerMove={handleCanvasPointerMove}
          onPointerUp={handleCanvasPointerEnd}
          onPointerCancel={handleCanvasPointerEnd}
          onDoubleClick={resetView}
          onWheel={handleCanvasWheel}
        />
        {loadState !== 'ready' && (
          <div className="simulator-panel__empty">
            <span title={loadState === 'error' ? errorDetail : undefined}>{statusLabel}</span>
            {loadState === 'error' && <small>{errorDetail}</small>}
            {loadState === 'empty' && (
              <Button size="sm" variant="primary" onClick={() => void buildCreasePattern()}>
                Build
              </Button>
            )}
          </div>
        )}
      </div>
      <div className="simulator-controls">
        <div className="simulator-transport" aria-label="Simulation controls">
          <IconButton
            size="sm"
            title="Refresh"
            tooltipSide="top"
            onClick={() => {
              clearPlayback();
              clearSettling();
              setPlaying(false);
              setModelError(null);
              void refreshFoldArtifacts();
            }}
            disabled={creaseCount === 0}
          >
            <RefreshCw size={14} />
          </IconButton>
          <IconButton
            size="sm"
            title={playing ? 'Pause' : 'Play'}
            tooltipSide="top"
            onClick={() => setPlaying((value) => !value)}
            disabled={loadState !== 'ready'}
          >
            {playing ? <Pause size={14} /> : <Play size={14} />}
          </IconButton>
          <IconButton
            size="sm"
            title="Step"
            tooltipSide="top"
            onClick={stepFoldTarget}
            disabled={loadState !== 'ready'}
          >
            <StepForward size={14} />
          </IconButton>
          <IconButton
            size="sm"
            title="Reset"
            tooltipSide="top"
            onClick={replayFromFlat}
            disabled={loadState !== 'ready'}
          >
            <RotateCcw size={14} />
          </IconButton>
        </div>
        <label className="simulator-slider">
          <span>Fold</span>
          <input
            aria-label="Fold percent"
            type="range"
            min="0"
            max="100"
            step="1"
            value={Math.round(foldPercent)}
            onChange={(event) => setFoldTarget(Number(event.currentTarget.value))}
            disabled={loadState !== 'ready'}
          />
          <output>{Math.round(foldPercent)}%</output>
        </label>
        <div className="simulator-readout">
          <span>{statusLabel}</span>
          <span>Step {step}</span>
          <span>Strain {strain.toFixed(4)}</span>
        </div>
      </div>
    </section>
  );
}

function maxPositionDelta(previous: Float32Array, next: Float32Array): number {
  let max = 0;
  for (let index = 0; index < Math.min(previous.length, next.length); index += 1) {
    max = Math.max(max, Math.abs((next[index] ?? 0) - (previous[index] ?? 0)));
  }
  return max;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function shortStatus(message: string): string {
  const trimmed = message.trim();
  if (!trimmed) return 'Simulator unavailable';
  const sentence = trimmed.split(/[.;]\s+/u)[0] ?? trimmed;
  return sentence.length > 54 ? `${sentence.slice(0, 51)}...` : sentence;
}

function drawFrame(
  canvas: HTMLCanvasElement,
  model: PreparedOrigamiModel,
  frame: SimulationFrame,
  view: SimulatorView,
  settings: SimulatorViewSettings
): void {
  const rect = canvas.getBoundingClientRect();
  const dpr = Math.max(1, window.devicePixelRatio || 1);
  const width = Math.max(360, Math.floor((rect.width || 720) * dpr));
  const height = Math.max(360, Math.floor((rect.height || 720) * dpr));
  if (canvas.width !== width || canvas.height !== height) {
    canvas.width = width;
    canvas.height = height;
  }

  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  ctx.clearRect(0, 0, width, height);
  ctx.fillStyle = '#0c0f12';
  ctx.fillRect(0, 0, width, height);

  const projected = projectPositions(frame.positions, view);
  const padding = Math.max(28, Math.min(width, height) * 0.08);
  const availableSize = Math.max(1, Math.min(width, height) - padding * 2);
  const scale = (availableSize / (2 * boundsRadius(frame.positions))) * view.zoom;
  const map = (point: ProjectedPoint) => ({
    x: width / 2 + point.x * scale,
    y: height / 2 - point.y * scale,
  });

  const triangles = triangleOrder(model.indices, projected);
  const faceAlpha = settings.renderMode === 'xray' ? 0.48 : 1;
  const surfaceEdgeAlpha = settings.renderMode === 'xray' ? 0.5 : 0.92;

  for (const triangle of triangles) {
    if (settings.showFaces) {
      const a = map(projected[triangle.vertices[0]] ?? { x: 0, y: 0, depth: 0 });
      const b = map(projected[triangle.vertices[1]] ?? { x: 0, y: 0, depth: 0 });
      const c = map(projected[triangle.vertices[2]] ?? { x: 0, y: 0, depth: 0 });
      ctx.beginPath();
      ctx.moveTo(a.x, a.y);
      ctx.lineTo(b.x, b.y);
      ctx.lineTo(c.x, c.y);
      ctx.closePath();
      ctx.fillStyle = triangleColor(frame.colors, triangle.vertices, faceAlpha);
      ctx.fill();
    }
    if (settings.showEdges && settings.showFaces) {
      drawTriangleEdges(ctx, model, triangle, projected, map, dpr, surfaceEdgeAlpha);
    }
  }

  if (settings.showEdges && (!settings.showFaces || settings.showHiddenLines)) {
    drawAllEdges(
      ctx,
      model,
      projected,
      map,
      dpr,
      settings.showFaces ? 0.34 : 0.95,
      settings.showFaces && settings.renderMode === 'paper'
    );
  }
}

function projectPositions(positions: Float32Array, view: SimulatorView): ProjectedPoint[] {
  const center = boundsCenter(positions);
  const points: ProjectedPoint[] = [];
  const cosYaw = Math.cos(view.yaw);
  const sinYaw = Math.sin(view.yaw);
  const cosPitch = Math.cos(view.pitch);
  const sinPitch = Math.sin(view.pitch);

  for (let index = 0; index < positions.length; index += 3) {
    const dx = (positions[index] ?? 0) - center.x;
    const dy = (positions[index + 1] ?? 0) - center.y;
    const dz = (positions[index + 2] ?? 0) - center.z;
    const yawX = cosYaw * dx + sinYaw * dz;
    const yawZ = -sinYaw * dx + cosYaw * dz;
    points.push({
      x: yawX,
      y: cosPitch * yawZ - sinPitch * dy,
      depth: sinPitch * yawZ + cosPitch * dy,
    });
  }
  return points;
}

function boundsCenter(positions: Float32Array): { x: number; y: number; z: number } {
  let minX = Infinity;
  let minY = Infinity;
  let minZ = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;
  let maxZ = -Infinity;
  for (let index = 0; index < positions.length; index += 3) {
    const x = positions[index] ?? 0;
    const y = positions[index + 1] ?? 0;
    const z = positions[index + 2] ?? 0;
    minX = Math.min(minX, x);
    minY = Math.min(minY, y);
    minZ = Math.min(minZ, z);
    maxX = Math.max(maxX, x);
    maxY = Math.max(maxY, y);
    maxZ = Math.max(maxZ, z);
  }
  if (!Number.isFinite(minX)) return { x: 0, y: 0, z: 0 };
  return {
    x: (minX + maxX) / 2,
    y: (minY + maxY) / 2,
    z: (minZ + maxZ) / 2,
  };
}

function boundsRadius(positions: Float32Array): number {
  const center = boundsCenter(positions);
  let radius = 0;
  for (let index = 0; index < positions.length; index += 3) {
    radius = Math.max(
      radius,
      Math.hypot(
        (positions[index] ?? 0) - center.x,
        (positions[index + 1] ?? 0) - center.y,
        (positions[index + 2] ?? 0) - center.z
      )
    );
  }
  return Math.max(0.001, radius);
}

interface OrderedTriangle {
  faceIndex: number;
  vertices: [number, number, number];
}

function triangleOrder(indices: Uint32Array, projected: ProjectedPoint[]): OrderedTriangle[] {
  const triangles: OrderedTriangle[] = [];
  for (let index = 0; index < indices.length; index += 3) {
    triangles.push({
      faceIndex: Math.floor(index / 3),
      vertices: [indices[index] ?? 0, indices[index + 1] ?? 0, indices[index + 2] ?? 0],
    });
  }
  return triangles.sort((a, b) => averageDepth(a, projected) - averageDepth(b, projected));
}

function averageDepth(triangle: OrderedTriangle, projected: ProjectedPoint[]): number {
  return triangle.vertices.reduce((total, vertex) => total + (projected[vertex]?.depth ?? 0), 0) / 3;
}

function triangleColor(colors: Float32Array, triangle: number[], alpha = 1): string {
  const channel = (offset: number) =>
    triangle.reduce((total, vertex) => total + (colors[vertex * 3 + offset] ?? 0.75), 0) / 3;
  const r = Math.round(channel(0) * 255);
  const g = Math.round(channel(1) * 255);
  const b = Math.round(channel(2) * 255);
  return alpha >= 1 ? `rgb(${r} ${g} ${b})` : `rgb(${r} ${g} ${b} / ${alpha})`;
}

function drawTriangleEdges(
  ctx: CanvasRenderingContext2D,
  model: PreparedOrigamiModel,
  triangle: OrderedTriangle,
  projected: ProjectedPoint[],
  map: (point: ProjectedPoint) => { x: number; y: number },
  dpr: number,
  alpha: number
): void {
  const faceEdges = model.facesEdges[triangle.faceIndex] ?? [];
  const pairs: Array<[number, number]> = [
    [triangle.vertices[0], triangle.vertices[1]],
    [triangle.vertices[1], triangle.vertices[2]],
    [triangle.vertices[2], triangle.vertices[0]],
  ];
  ctx.setLineDash([]);
  ctx.lineWidth = Math.max(1.2, dpr * 1.05);
  pairs.forEach(([from, to], side) => {
    drawEdgeSegment(
      ctx,
      model,
      projected,
      map,
      from,
      to,
      faceEdges[side] ?? findEdge(model.edgesVertices, from, to),
      alpha
    );
  });
}

function drawAllEdges(
  ctx: CanvasRenderingContext2D,
  model: PreparedOrigamiModel,
  projected: ProjectedPoint[],
  map: (point: ProjectedPoint) => { x: number; y: number },
  dpr: number,
  alpha: number,
  dashed: boolean
): void {
  ctx.setLineDash(dashed ? [Math.max(3, dpr * 3), Math.max(3, dpr * 3)] : []);
  ctx.lineWidth = Math.max(1.5, dpr * 1.25);
  model.edgesVertices.forEach((edge, index) => {
    drawEdgeSegment(ctx, model, projected, map, edge[0], edge[1], index, alpha);
  });
  ctx.setLineDash([]);
}

function drawEdgeSegment(
  ctx: CanvasRenderingContext2D,
  model: PreparedOrigamiModel,
  projected: ProjectedPoint[],
  map: (point: ProjectedPoint) => { x: number; y: number },
  from: number,
  to: number,
  edgeIndex: number,
  alpha: number
): void {
  const a = map(projected[from] ?? { x: 0, y: 0, depth: 0 });
  const b = map(projected[to] ?? { x: 0, y: 0, depth: 0 });
  ctx.beginPath();
  ctx.moveTo(a.x, a.y);
  ctx.lineTo(b.x, b.y);
  ctx.strokeStyle = edgeColor(model.edgesAssignment[edgeIndex], alpha);
  ctx.stroke();
}

function findEdge(edges: [number, number][], from: number, to: number): number {
  return edges.findIndex(
    (edge) => (edge[0] === from && edge[1] === to) || (edge[0] === to && edge[1] === from)
  );
}

function edgeColor(assignment: string | undefined, alpha = 1): string {
  if (assignment === 'M') return `rgb(224 108 117 / ${alpha})`;
  if (assignment === 'V') return `rgb(95 179 165 / ${alpha})`;
  if (assignment === 'B') return `rgb(17 20 23 / ${alpha})`;
  if (assignment === 'F') return `rgb(232 237 240 / ${alpha * 0.55})`;
  return `rgb(232 237 240 / ${alpha * 0.32})`;
}
