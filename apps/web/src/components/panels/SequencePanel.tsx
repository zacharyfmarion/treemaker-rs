import { useMemo } from 'react';
import { AlertTriangle, ArrowRight, CheckCircle2, CircleDashed, Layers3, Play } from 'lucide-react';
import type {
  FoldDocument,
  SequenceInstructionStep,
  SequencePlan,
  SequenceStateSnapshot,
  SequenceTargetState,
} from '../../engine/types';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { Button } from '../ui/Button';

const PREVIEW_VIEWBOX = 320;
const PREVIEW_PADDING = 24;

export function SequencePanel() {
  const foldArtifacts = useWorkspaceStore((state) => state.foldArtifacts);
  const foldArtifactError = useWorkspaceStore((state) => state.foldArtifactError);
  const sequencePlan = useWorkspaceStore((state) => state.sequencePlan);
  const sequenceTarget = useWorkspaceStore((state) => state.sequenceTarget);
  const sequencePlanning = useWorkspaceStore((state) => state.sequencePlanning);
  const sequenceError = useWorkspaceStore((state) => state.sequenceError);
  const planFoldingSequence = useWorkspaceStore((state) => state.planFoldingSequence);

  const statusTone =
    sequenceError || sequencePlan?.status === 'unsupported'
      ? 'bad'
      : sequencePlan?.status === 'complete'
        ? 'good'
        : sequencePlan
          ? 'warn'
          : 'warn';
  const statusLabel = sequenceError
    ? sequenceError
    : sequencePlan
      ? formatStatus(sequencePlan.status)
      : foldArtifacts
        ? 'Sequence not planned'
        : foldArtifactError || 'Crease pattern pending';
  const headerSummary = sequencePlan
    ? `${formatStatus(sequencePlan.status)} | ${sequencePlan.steps.length} step${sequencePlan.steps.length === 1 ? '' : 's'}`
    : statusLabel;

  return (
    <section className="panel-shell sequence-panel">
      <div className="panel-toolbar">
        <div className="panel-toolbar__group">
          <span className="panel-title">Sequence</span>
          <span className="sequence-panel__toolbar-summary" data-tone={statusTone}>
            {headerSummary}
          </span>
        </div>
        <Button
          size="sm"
          variant="secondary"
          disabled={sequencePlanning || !foldArtifacts}
          onClick={() => void planFoldingSequence()}
        >
          <Play size={14} />
          {sequencePlanning ? 'Planning' : 'Plan'}
        </Button>
      </div>
      <div className="panel-body sequence-panel__body">
        <SequenceDetails
          sequencePlan={sequencePlan}
          sequenceTarget={sequenceTarget}
          statusTone={statusTone}
          statusLabel={statusLabel}
        />
        {sequencePlan && (
          <SequenceDiagramList plan={sequencePlan} />
        )}
      </div>
    </section>
  );
}

function SequenceDetails({
  sequencePlan,
  sequenceTarget,
  statusTone,
  statusLabel,
}: {
  sequencePlan: SequencePlan | null;
  sequenceTarget: SequenceTargetState | null;
  statusTone: 'good' | 'warn' | 'bad';
  statusLabel: string;
}) {
  return (
    <details className="sequence-panel__details">
      <summary>
        <span>Details</span>
        <span>{sequencePlan ? formatStatus(sequencePlan.status) : statusLabel}</span>
      </summary>
      <div className="metric-grid sequence-panel__metrics">
        <Metric label="Status" value={sequencePlan ? formatStatus(sequencePlan.status) : 'Idle'} />
        <Metric label="Steps" value={sequencePlan?.steps.length ?? 0} />
        <Metric label="Open" value={sequencePlan?.search.best_unresolved_creases ?? 0} />
        <Metric label="States" value={sequencePlan?.search.states_explored ?? 0} />
      </div>
      <div className="status-row" data-tone={statusTone}>
        {statusTone === 'good' ? <CheckCircle2 size={15} /> : <AlertTriangle size={15} />}
        <span>{statusLabel}</span>
      </div>
      {sequenceTarget && (
        <div className="status-row" data-tone="good">
          <CheckCircle2 size={15} />
          <span>
            {sequenceTarget.normalized.faces_vertices.length} faces, {sequenceTarget.states} layer
            state{sequenceTarget.states === '1' ? '' : 's'}
          </span>
        </div>
      )}
      {sequencePlan?.search.budget_exhausted && (
        <div className="status-row" data-tone="warn">
          <CircleDashed size={15} />
          <span>Search budget reached with a partial result</span>
        </div>
      )}
      {sequencePlan?.diagnostics.slice(0, 4).map((diagnostic) => (
        <div
          key={`${diagnostic.code}:${diagnostic.message}`}
          className="status-row"
          data-tone={diagnostic.severity === 'error' ? 'bad' : 'warn'}
        >
          <CircleDashed size={15} />
          <span>{diagnostic.message}</span>
        </div>
      ))}
    </details>
  );
}

function SequenceDiagramList({ plan }: { plan: SequencePlan }) {
  const stateById = useMemo(
    () => new Map(plan.states.map((state) => [state.id, state])),
    [plan.states]
  );

  if (plan.steps.length === 0) {
    return (
      <ol className="sequence-panel__steps">
        <li className="sequence-panel__empty-step">No sequence steps</li>
      </ol>
    );
  }

  return (
    <ol className="sequence-panel__steps" aria-label="Folding sequence diagram">
      {plan.steps.map((step, index) => {
        const beforeState = step.before_state ? stateById.get(step.before_state) : null;
        const afterState = step.after_state ? stateById.get(step.after_state) : null;
        const highlights = highlightsForStep(step);
        return (
          <li key={step.id} className="sequence-diagram-step">
            <div className="sequence-diagram-step__header">
              <div>
                <span>Step {index + 1}</span>
                <strong>{formatKind(step.kind)}</strong>
              </div>
              <span>
                {stepCreaseCount(step)} crease{stepCreaseCount(step) === 1 ? '' : 's'}
              </span>
            </div>
            <div className="sequence-diagram-step__visuals">
              <SequencePreview
                title="Before"
                state={beforeState}
                mode="folded"
                highlights={highlights}
                stepLabel={`Step ${index + 1}`}
              />
              <div className="sequence-diagram-step__arrow" aria-hidden="true">
                <ArrowRight size={17} />
              </div>
              <SequencePreview
                title="After"
                state={afterState}
                mode="folded"
                highlights={highlights}
                stepLabel={`Step ${index + 1}`}
              />
            </div>
            <div className="sequence-diagram-step__copy">
              <div className="sequence-diagram-step__label">{step.label}</div>
              <div className="sequence-diagram-step__meta">
                {step.after_state ? `${step.before_state} to ${step.after_state}` : formatKind(step.kind)}
              </div>
            </div>
          </li>
        );
      })}
    </ol>
  );
}

function SequencePreview({
  title,
  state,
  mode,
  highlights,
  stepLabel,
}: {
  title: string;
  state: SequenceStateSnapshot | null | undefined;
  mode: 'paper' | 'folded';
  highlights: SequenceHighlights;
  stepLabel?: string;
}) {
  const projection = useMemo(() => {
    if (!state) return null;
    return createPreviewProjection(pointsForState(state, mode));
  }, [mode, state]);

  if (!state || !projection) {
    return (
      <div className="sequence-panel__preview" data-empty>
        <div className="sequence-panel__preview-title">
          <Layers3 size={13} />
          <span>{title}</span>
        </div>
        <div className="sequence-panel__preview-empty">State unavailable</div>
      </div>
    );
  }

  const points = pointsForState(state, mode);

  return (
    <div className="sequence-panel__preview">
      <div className="sequence-panel__preview-title">
        <Layers3 size={13} />
        <span>{title}</span>
        <span>{state.id}</span>
      </div>
      <svg
        className="sequence-preview-canvas"
        viewBox={`0 0 ${PREVIEW_VIEWBOX} ${PREVIEW_VIEWBOX}`}
        role="img"
        aria-label={[stepLabel, title, mode === 'folded' ? 'folded state' : 'crease pattern', state.id]
          .filter(Boolean)
          .join(' ')}
      >
        <rect className="sequence-preview-plane" x="12" y="12" width="296" height="296" rx="4" />
        {state.document.faces_vertices.map((face, index) => {
          const polygon = polygonPoints(face, points, projection);
          if (!polygon) return null;
          return (
            <polygon
              key={`face-${index}`}
              className={[
                'sequence-preview-face',
                highlights.faces.has(index) ? 'sequence-preview-face--highlight' : '',
              ].join(' ')}
              points={polygon}
            />
          );
        })}
        {state.document.edges_vertices.map(([a, b], index) => {
          const p1 = projection(points[a]);
          const p2 = projection(points[b]);
          if (!p1 || !p2) return null;
          return (
            <line
              key={`edge-${index}`}
              className={[
                'sequence-preview-crease',
                `sequence-preview-crease--${assignmentForEdge(state.document, index).toLowerCase()}`,
                highlights.creases.has(index) ? 'sequence-preview-crease--highlight' : '',
              ].join(' ')}
              x1={p1.x}
              y1={p1.y}
              x2={p2.x}
              y2={p2.y}
            />
          );
        })}
      </svg>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: number | string }) {
  return (
    <div className="metric">
      <div className="metric__label">{label}</div>
      <div className="metric__value">{value}</div>
    </div>
  );
}

function formatStatus(status: string): string {
  return status.replaceAll('_', ' ');
}

function formatKind(kind: string): string {
  return kind.replaceAll('_', ' ');
}

function stepCreaseCount(step: SequenceInstructionStep): number {
  const region = (step as { region?: { creases: number[] } }).region;
  if (step.kind === 'unsupported_region' && region) return region.creases.length;
  return step.affected_creases?.length ?? 0;
}

interface SequenceHighlights {
  creases: Set<number>;
  faces: Set<number>;
}

function highlightsForStep(step: SequenceInstructionStep): SequenceHighlights {
  const region = (step as { region?: { creases: number[]; faces: number[] } }).region;
  return {
    creases: new Set(region?.creases ?? step.affected_creases ?? []),
    faces: new Set(region?.faces ?? step.affected_faces ?? []),
  };
}

function pointsForState(state: SequenceStateSnapshot, mode: 'paper' | 'folded'): Array<[number, number]> {
  if (mode === 'folded' && state.folded_vertices.length === state.document.vertices_coords.length) {
    return state.folded_vertices;
  }
  return state.document.vertices_coords.map((coord) => [coord[0] ?? 0, coord[1] ?? 0]);
}

function createPreviewProjection(points: Array<[number, number]>) {
  if (points.length === 0) return null;
  const bounds = points.reduce(
    (acc, [x, y]) => ({
      minX: Math.min(acc.minX, x),
      maxX: Math.max(acc.maxX, x),
      minY: Math.min(acc.minY, y),
      maxY: Math.max(acc.maxY, y),
    }),
    { minX: Infinity, maxX: -Infinity, minY: Infinity, maxY: -Infinity }
  );
  const minX = Number.isFinite(bounds.minX) ? bounds.minX : 0;
  const maxX = Number.isFinite(bounds.maxX) ? bounds.maxX : 1;
  const minY = Number.isFinite(bounds.minY) ? bounds.minY : 0;
  const maxY = Number.isFinite(bounds.maxY) ? bounds.maxY : 1;
  const spanX = Math.max(0.001, maxX - minX);
  const spanY = Math.max(0.001, maxY - minY);
  const scale = Math.min(
    (PREVIEW_VIEWBOX - PREVIEW_PADDING * 2) / spanX,
    (PREVIEW_VIEWBOX - PREVIEW_PADDING * 2) / spanY
  );
  const offsetX = (PREVIEW_VIEWBOX - spanX * scale) / 2;
  const offsetY = (PREVIEW_VIEWBOX - spanY * scale) / 2;
  return (point: [number, number] | undefined) => {
    if (!point) return null;
    const [x, y] = point;
    return {
      x: offsetX + (x - minX) * scale,
      y: PREVIEW_VIEWBOX - offsetY - (y - minY) * scale,
    };
  };
}

function polygonPoints(
  face: number[],
  points: Array<[number, number]>,
  project: NonNullable<ReturnType<typeof createPreviewProjection>>
): string | null {
  const projected = face
    .map((vertex) => project(points[vertex]))
    .filter((point): point is { x: number; y: number } => point !== null);
  if (projected.length < 3) return null;
  return projected.map((point) => `${point.x},${point.y}`).join(' ');
}

function assignmentForEdge(document: FoldDocument, edge: number): string {
  return document.edges_assignment?.[edge] ?? 'U';
}
