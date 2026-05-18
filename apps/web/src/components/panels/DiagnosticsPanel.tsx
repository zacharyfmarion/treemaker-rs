import { AlertTriangle, CheckCircle2, CircleDashed } from 'lucide-react';
import { useWorkspaceStore } from '../../store/workspaceStore';

export function DiagnosticsPanel() {
  const project = useWorkspaceStore((state) => state.project);
  const status = useWorkspaceStore((state) => state.status);
  const engineReady = useWorkspaceStore((state) => state.engineReady);
  const error = useWorkspaceStore((state) => state.error);
  const lastOptimization = useWorkspaceStore((state) => state.lastOptimization);

  const cpReady = project.creases.length > 0 && project.facets.length > 0;
  const optimizationTone = lastOptimization?.is_feasible
    ? 'good'
    : lastOptimization
      ? 'bad'
      : 'warn';
  const engineIcon = error ? (
    <AlertTriangle size={15} />
  ) : engineReady ? (
    <CheckCircle2 size={15} />
  ) : (
    <CircleDashed size={15} />
  );

  return (
    <section className="panel-shell">
      <div className="panel-toolbar">
        <span className="panel-title">Diagnostics</span>
      </div>
      <div className="panel-body">
        <div className="metric-grid">
          <Metric label="Nodes" value={project.nodes.length} />
          <Metric label="Edges" value={project.edges.length} />
          <Metric label="Paths" value={project.paths.length} />
          <Metric label="Conditions" value={project.conditions.length} />
        </div>
        <div className="status-row" data-tone={error ? 'bad' : engineReady ? 'good' : 'warn'}>
          {engineIcon}
          <span>{error ? error.message : engineReady ? 'Engine ready' : 'Loading engine'}</span>
        </div>
        <div className="status-row" data-tone={optimizationTone}>
          {lastOptimization?.is_feasible ? <CheckCircle2 size={15} /> : <CircleDashed size={15} />}
          <span>
            {lastOptimization
              ? lastOptimization.message
              : status === 'optimizing'
                ? 'Optimizing scale'
                : 'Optimization pending'}
          </span>
        </div>
        <div className="status-row" data-tone={cpReady ? 'good' : 'warn'}>
          {cpReady ? <CheckCircle2 size={15} /> : <CircleDashed size={15} />}
          <span>
            {cpReady
              ? `${project.creases.length} creases, ${project.facets.length} facets`
              : status === 'building_crease_pattern'
                ? 'Building crease pattern'
                : 'Crease pattern pending'}
          </span>
        </div>
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
