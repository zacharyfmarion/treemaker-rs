import { AlertTriangle, CheckCircle2, CircleDashed } from 'lucide-react';
import { useWorkspaceStore } from '../../store/workspaceStore';

export function DiagnosticsPanel() {
  const project = useWorkspaceStore((state) => state.project);
  const documentMode = useWorkspaceStore((state) => state.documentMode);
  const importedCreasePattern = useWorkspaceStore((state) => state.importedCreasePattern);
  const status = useWorkspaceStore((state) => state.status);
  const engineReady = useWorkspaceStore((state) => state.engineReady);
  const error = useWorkspaceStore((state) => state.error);
  const lastOptimization = useWorkspaceStore((state) => state.lastOptimization);

  const cpReady = project.creases.length > 0 && project.facets.length > 0;
  const infeasibleConditions = project.conditions.filter((condition) => !condition.isFeasible);
  const conditionedNodes = project.nodes.filter((node) => node.isConditioned).length;
  const conditionedEdges = project.edges.filter((edge) => edge.isConditioned).length;
  const conditionedPaths = project.paths.filter((path) => path.isConditioned).length;
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

  if (documentMode === 'crease-pattern') {
    const diagnostics = importedCreasePattern?.diagnostics;
    const hasErrors = Boolean(diagnostics?.errors.length);
    const hasWarnings = Boolean(diagnostics?.warnings.length);
    return (
      <section className="panel-shell">
        <div className="panel-toolbar">
          <span className="panel-title">Diagnostics</span>
        </div>
        <div className="panel-body">
          <div className="metric-grid">
            <Metric label="Vertices" value={importedCreasePattern?.stats.vertices ?? 0} />
            <Metric label="Edges" value={importedCreasePattern?.stats.edges ?? 0} />
            <Metric label="Faces" value={importedCreasePattern?.stats.faces ?? 0} />
            <Metric label="Mode" value="CP-only" />
          </div>
          <div className="status-row" data-tone={hasErrors ? 'bad' : hasWarnings ? 'warn' : 'good'}>
            {hasErrors ? <AlertTriangle size={15} /> : <CheckCircle2 size={15} />}
            <span>
              {hasErrors
                ? diagnostics?.errors[0]
                : hasWarnings
                  ? diagnostics?.warnings[0]
                  : 'Imported crease pattern ready'}
            </span>
          </div>
          {importedCreasePattern?.selectedFrame && (
            <div className="status-row" data-tone="good">
              <CheckCircle2 size={15} />
              <span>
                FOLD frame {importedCreasePattern.selectedFrame.index}
                {importedCreasePattern.selectedFrame.title
                  ? `: ${importedCreasePattern.selectedFrame.title}`
                  : ''}
              </span>
            </div>
          )}
          <div
            className="status-row"
            data-tone={importedCreasePattern?.simulationModelError ? 'warn' : 'good'}
          >
            {importedCreasePattern?.simulationModelError ? (
              <CircleDashed size={15} />
            ) : (
              <CheckCircle2 size={15} />
            )}
            <span>
              {importedCreasePattern?.simulationModelError ??
                'Simulator-ready topology available'}
            </span>
          </div>
        </div>
      </section>
    );
  }

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
        <div className="status-row" data-tone={infeasibleConditions.length === 0 ? 'good' : 'bad'}>
          {infeasibleConditions.length === 0 ? <CheckCircle2 size={15} /> : <AlertTriangle size={15} />}
          <span>
            {infeasibleConditions.length === 0
              ? 'All conditions feasible'
              : `${infeasibleConditions.length} infeasible condition${infeasibleConditions.length === 1 ? '' : 's'}: ${infeasibleConditions
                  .slice(0, 3)
                  .map((condition) => condition.id)
                  .join(', ')}`}
          </span>
        </div>
        <div className="status-row" data-tone="warn">
          <CircleDashed size={15} />
          <span>
            Conditioned parts: {conditionedNodes} nodes, {conditionedEdges} edges, {conditionedPaths} paths
          </span>
        </div>
        {lastOptimization && !lastOptimization.is_feasible && (
          <div className="status-row" data-tone="bad">
            <AlertTriangle size={15} />
            <span>Optimizer reported an infeasible constrained system</span>
          </div>
        )}
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
