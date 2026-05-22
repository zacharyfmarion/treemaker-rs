import { AlertTriangle, CheckCircle2, CircleDashed, Play } from 'lucide-react';
import type { SequenceInstructionStep } from '../../engine/types';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { Button } from '../ui/Button';

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

  return (
    <section className="panel-shell sequence-panel">
      <div className="panel-toolbar">
        <div className="panel-toolbar__group">
          <span className="panel-title">Sequence</span>
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
        <div className="metric-grid">
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
        {sequencePlan && (
          <ol className="sequence-panel__steps">
            {sequencePlan.steps.map((step) => (
              <li key={step.id} className="sequence-panel__step">
                <div className="sequence-panel__step-title">
                  <span>{step.label}</span>
                  <span>{formatKind(step.kind)}</span>
                </div>
                <div className="sequence-panel__step-meta">
                  {stepCreaseCount(step)} creases
                  {step.after_state ? ` | ${step.before_state} -> ${step.after_state}` : ''}
                </div>
              </li>
            ))}
          </ol>
        )}
      </div>
    </section>
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
