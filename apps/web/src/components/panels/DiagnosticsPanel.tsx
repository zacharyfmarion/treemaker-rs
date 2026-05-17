import { AlertTriangle, CheckCircle2, CircleDashed } from 'lucide-react';
import { useWorkspaceStore } from '../../store/workspaceStore';

export function DiagnosticsPanel() {
  const project = useWorkspaceStore((state) => state.project);

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
          <Metric label="Creases" value={project.creases.length} />
        </div>
        <div className="status-row" data-tone="good">
          <CheckCircle2 size={15} />
          <span>Document ready</span>
        </div>
        <div className="status-row" data-tone="warn">
          <CircleDashed size={15} />
          <span>Optimization pending</span>
        </div>
        <div className="status-row" data-tone="warn">
          <AlertTriangle size={15} />
          <span>Crease pattern sample</span>
        </div>
      </div>
    </section>
  );
}

function Metric({ label, value }: { label: string; value: number }) {
  return (
    <div className="metric">
      <div className="metric__label">{label}</div>
      <div className="metric__value">{value}</div>
    </div>
  );
}
