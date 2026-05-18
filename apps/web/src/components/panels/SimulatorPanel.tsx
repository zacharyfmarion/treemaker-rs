import { useCallback, useEffect, useRef, useState } from 'react';
import {
  Pause,
  Play,
  RefreshCw,
  RotateCcw,
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
import { useWorkspaceStore } from '../../store/workspaceStore';
import { Button } from '../ui/Button';
import { IconButton } from '../ui/IconButton';

type LoadState = 'idle' | 'loading' | 'ready' | 'empty' | 'error';

export function SimulatorPanel() {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const controllerRef = useRef<OrigamiSimulatorController | null>(null);
  const modelRef = useRef<PreparedOrigamiModel | null>(null);
  const frameRef = useRef<SimulationFrame | null>(null);
  const rafRef = useRef<number | null>(null);
  const foldPercentRef = useRef(60);

  const creaseCount = useWorkspaceStore((state) => state.project.creases.length);
  const foldArtifacts = useWorkspaceStore((state) => state.foldArtifacts);
  const refreshFoldArtifacts = useWorkspaceStore((state) => state.refreshFoldArtifacts);
  const buildCreasePattern = useWorkspaceStore((state) => state.buildCreasePattern);

  const [foldPercent, setFoldPercent] = useState(60);
  const [playing, setPlaying] = useState(false);
  const [loadState, setLoadState] = useState<LoadState>('idle');
  const [step, setStep] = useState(0);
  const [strain, setStrain] = useState(0);
  const [modelStats, setModelStats] = useState({ vertices: 0, triangles: 0 });

  useEffect(() => {
    foldPercentRef.current = foldPercent;
  }, [foldPercent]);

  const drawCurrentFrame = useCallback(() => {
    const canvas = canvasRef.current;
    const model = modelRef.current;
    const frame = frameRef.current;
    if (!canvas || !model || !frame) return;
    drawFrame(canvas, model, frame);
    setStep(frame.step);
    setStrain(frame.diagnostics.maxEdgeStrain ?? 0);
  }, []);

  const stepSimulation = useCallback(
    (steps = 1) => {
      const controller = controllerRef.current;
      if (!controller) return;
      frameRef.current = controller.step(steps);
      drawCurrentFrame();
    },
    [drawCurrentFrame]
  );

  useEffect(() => {
    if (creaseCount === 0) {
      setLoadState('empty');
      return;
    }
    if (foldArtifacts) {
      setLoadState('ready');
      return;
    }

    let cancelled = false;
    setLoadState('loading');
    void refreshFoldArtifacts().then((artifacts) => {
      if (cancelled) return;
      setLoadState(artifacts ? 'ready' : 'error');
    });
    return () => {
      cancelled = true;
    };
  }, [creaseCount, foldArtifacts, refreshFoldArtifacts]);

  useEffect(() => {
    controllerRef.current?.dispose();
    controllerRef.current = null;
    modelRef.current = null;
    frameRef.current = null;
    setModelStats({ vertices: 0, triangles: 0 });

    if (!foldArtifacts) return;

    try {
      const model = prepareFoldModel(foldArtifacts.fold as SimulatorFoldDocument);
      const controller = createOrigamiSimulator({
        model,
        options: { foldPercent: foldPercentRef.current, damping: 0.34, stepsPerFrame: 6 },
      });
      modelRef.current = model;
      controllerRef.current = controller;
      setModelStats({ vertices: model.vertexCount, triangles: model.faceCount });
      frameRef.current = controller.step(20);
      setLoadState('ready');
      drawCurrentFrame();
    } catch (error) {
      console.warn('Failed to prepare simulator model', error);
      setModelStats({ vertices: 0, triangles: 0 });
      setLoadState('error');
    }

    return () => {
      controllerRef.current?.dispose();
      controllerRef.current = null;
      modelRef.current = null;
      frameRef.current = null;
      setModelStats({ vertices: 0, triangles: 0 });
    };
  }, [drawCurrentFrame, foldArtifacts]);

  useEffect(() => {
    controllerRef.current?.setFoldPercent(foldPercent);
    stepSimulation(12);
  }, [foldPercent, stepSimulation]);

  useEffect(() => {
    if (!playing || typeof window === 'undefined') return;
    const tick = () => {
      stepSimulation();
      rafRef.current = window.requestAnimationFrame(tick);
    };
    rafRef.current = window.requestAnimationFrame(tick);
    return () => {
      if (rafRef.current !== null) window.cancelAnimationFrame(rafRef.current);
      rafRef.current = null;
    };
  }, [playing, stepSimulation]);

  useEffect(() => {
    if (typeof ResizeObserver === 'undefined') return;
    const canvas = canvasRef.current;
    if (!canvas) return;
    const observer = new ResizeObserver(drawCurrentFrame);
    observer.observe(canvas);
    return () => observer.disconnect();
  }, [drawCurrentFrame]);

  const reset = () => {
    controllerRef.current?.reset();
    controllerRef.current?.setFoldPercent(foldPercent);
    frameRef.current = controllerRef.current?.step(20) ?? null;
    drawCurrentFrame();
  };

  const statusLabel =
    loadState === 'ready'
      ? `${modelStats.vertices} vertices | ${modelStats.triangles} triangles`
      : loadState === 'loading'
        ? 'Loading'
        : loadState === 'empty'
          ? 'No crease pattern'
          : loadState === 'error'
            ? 'Unavailable'
            : 'Idle';

  return (
    <section className="panel-shell simulator-panel">
      <div className="panel-toolbar">
        <div className="panel-toolbar__group">
          <Waves size={14} />
          <span className="panel-title">Simulator</span>
        </div>
        <div className="panel-toolbar__group">
          <IconButton
            size="sm"
            title="Refresh"
            tooltipSide="bottom"
            onClick={() => void refreshFoldArtifacts()}
            disabled={creaseCount === 0}
          >
            <RefreshCw size={14} />
          </IconButton>
          <IconButton
            size="sm"
            title={playing ? 'Pause' : 'Play'}
            tooltipSide="bottom"
            onClick={() => setPlaying((value) => !value)}
            disabled={loadState !== 'ready'}
          >
            {playing ? <Pause size={14} /> : <Play size={14} />}
          </IconButton>
          <IconButton
            size="sm"
            title="Step"
            tooltipSide="bottom"
            onClick={() => stepSimulation(8)}
            disabled={loadState !== 'ready'}
          >
            <StepForward size={14} />
          </IconButton>
          <IconButton
            size="sm"
            title="Reset"
            tooltipSide="bottom"
            onClick={reset}
            disabled={loadState !== 'ready'}
          >
            <RotateCcw size={14} />
          </IconButton>
        </div>
      </div>
      <div className="panel-body simulator-panel__body">
        <canvas
          ref={canvasRef}
          className="simulator-canvas"
          aria-label="Origami folded-base simulator"
        />
        {loadState !== 'ready' && (
          <div className="simulator-panel__empty">
            <span>{statusLabel}</span>
            {loadState === 'empty' && (
              <Button size="sm" variant="primary" onClick={() => void buildCreasePattern()}>
                Build
              </Button>
            )}
          </div>
        )}
      </div>
      <div className="simulator-controls">
        <label className="simulator-slider">
          <span>Fold</span>
          <input
            aria-label="Fold percent"
            type="range"
            min="-100"
            max="100"
            step="1"
            value={foldPercent}
            onChange={(event) => setFoldPercent(Number(event.currentTarget.value))}
            disabled={loadState !== 'ready'}
          />
          <output>{foldPercent}%</output>
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

function drawFrame(
  canvas: HTMLCanvasElement,
  model: PreparedOrigamiModel,
  frame: SimulationFrame
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

  const projected = projectPositions(frame.positions);
  const bounds = projected.reduce(
    (acc, point) => ({
      minX: Math.min(acc.minX, point.x),
      maxX: Math.max(acc.maxX, point.x),
      minY: Math.min(acc.minY, point.y),
      maxY: Math.max(acc.maxY, point.y),
    }),
    { minX: Infinity, maxX: -Infinity, minY: Infinity, maxY: -Infinity }
  );
  const spanX = Math.max(0.001, bounds.maxX - bounds.minX);
  const spanY = Math.max(0.001, bounds.maxY - bounds.minY);
  const padding = Math.max(28, Math.min(width, height) * 0.08);
  const scale = Math.min((width - padding * 2) / spanX, (height - padding * 2) / spanY);
  const map = (point: { x: number; y: number }) => ({
    x: padding + (point.x - bounds.minX) * scale,
    y: height - padding - (point.y - bounds.minY) * scale,
  });

  const triangles = triangleOrder(model.indices, frame.positions);
  for (const triangle of triangles) {
    const a = map(projected[triangle[0]] ?? { x: 0, y: 0 });
    const b = map(projected[triangle[1]] ?? { x: 0, y: 0 });
    const c = map(projected[triangle[2]] ?? { x: 0, y: 0 });
    ctx.beginPath();
    ctx.moveTo(a.x, a.y);
    ctx.lineTo(b.x, b.y);
    ctx.lineTo(c.x, c.y);
    ctx.closePath();
    ctx.fillStyle = triangleColor(frame.colors, triangle);
    ctx.fill();
    ctx.strokeStyle = 'rgba(16, 20, 23, 0.42)';
    ctx.lineWidth = Math.max(1, dpr);
    ctx.stroke();
  }

  ctx.lineWidth = Math.max(1.5, dpr * 1.25);
  model.edgesVertices.forEach((edge, index) => {
    const a = map(projected[edge[0]] ?? { x: 0, y: 0 });
    const b = map(projected[edge[1]] ?? { x: 0, y: 0 });
    ctx.beginPath();
    ctx.moveTo(a.x, a.y);
    ctx.lineTo(b.x, b.y);
    ctx.strokeStyle = edgeColor(model.edgesAssignment[index]);
    ctx.stroke();
  });
}

function projectPositions(positions: Float32Array): Array<{ x: number; y: number }> {
  const points: Array<{ x: number; y: number }> = [];
  for (let index = 0; index < positions.length; index += 3) {
    points.push({
      x: positions[index] ?? 0,
      y: (positions[index + 2] ?? 0) - (positions[index + 1] ?? 0) * 0.38,
    });
  }
  return points;
}

function triangleOrder(indices: Uint32Array, positions: Float32Array): number[][] {
  const triangles: number[][] = [];
  for (let index = 0; index < indices.length; index += 3) {
    triangles.push([indices[index] ?? 0, indices[index + 1] ?? 0, indices[index + 2] ?? 0]);
  }
  return triangles.sort((a, b) => averageHeight(a, positions) - averageHeight(b, positions));
}

function averageHeight(triangle: number[], positions: Float32Array): number {
  return triangle.reduce((total, vertex) => total + (positions[vertex * 3 + 1] ?? 0), 0) / 3;
}

function triangleColor(colors: Float32Array, triangle: number[]): string {
  const channel = (offset: number) =>
    triangle.reduce((total, vertex) => total + (colors[vertex * 3 + offset] ?? 0.75), 0) / 3;
  const r = Math.round(channel(0) * 255);
  const g = Math.round(channel(1) * 255);
  const b = Math.round(channel(2) * 255);
  return `rgb(${r} ${g} ${b})`;
}

function edgeColor(assignment: string | undefined): string {
  if (assignment === 'M') return '#e06c75';
  if (assignment === 'V') return '#5fb3a5';
  if (assignment === 'B') return '#111417';
  if (assignment === 'F') return 'rgba(232, 237, 240, 0.55)';
  return 'rgba(232, 237, 240, 0.32)';
}
