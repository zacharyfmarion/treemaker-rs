import { Activity, Circle, GitBranch, Square } from 'lucide-react';
import { formatNumber } from '../../lib/geometry';
import { useWorkspaceStore } from '../../store/workspaceStore';

export function InspectorPanel() {
  const project = useWorkspaceStore((state) => state.project);
  const selection = useWorkspaceStore((state) => state.selection);

  const selectedNode =
    selection.kind === 'node' ? project.nodes.find((node) => node.id === selection.id) : null;
  const selectedEdge =
    selection.kind === 'edge' ? project.edges.find((edge) => edge.id === selection.id) : null;
  const selectedCrease =
    selection.kind === 'crease' ? project.creases.find((crease) => crease.id === selection.id) : null;

  return (
    <section className="panel-shell inspector-panel">
      <div className="panel-toolbar">
        <span className="panel-title">Inspector</span>
      </div>
      <div className="panel-body">
        {selectedNode && (
          <>
            <div className="inspector-heading"><Circle size={15} /> Node {selectedNode.id}</div>
            <Row label="Label" value={selectedNode.label} />
            <Row label="X" value={formatNumber(selectedNode.loc.x)} />
            <Row label="Y" value={formatNumber(selectedNode.loc.y)} />
            <Row label="Leaf" value={selectedNode.isLeaf ? 'Yes' : 'No'} />
            <Row label="Pinned" value={selectedNode.isPinned ? 'Yes' : 'No'} />
          </>
        )}
        {selectedEdge && (
          <>
            <div className="inspector-heading"><GitBranch size={15} /> Edge {selectedEdge.id}</div>
            <Row label="Label" value={selectedEdge.label} />
            <Row label="Nodes" value={selectedEdge.nodes.join(' -> ')} />
            <Row label="Length" value={formatNumber(selectedEdge.length)} />
            <Row label="Strain" value={formatNumber(selectedEdge.strain)} />
            <Row label="Stiffness" value={formatNumber(selectedEdge.stiffness)} />
          </>
        )}
        {selectedCrease && (
          <>
            <div className="inspector-heading"><Activity size={15} /> Crease {selectedCrease.id}</div>
            <Row label="Fold" value={selectedCrease.fold} />
            <Row label="Kind" value={selectedCrease.kind} />
          </>
        )}
        {selection.kind === 'tree' && (
          <>
            <div className="inspector-heading"><Square size={15} /> Tree</div>
            <Row label="Title" value={project.title} />
            <Row label="Paper" value={`${project.paper.width} x ${project.paper.height}`} />
            <Row label="Scale" value={formatNumber(project.scale)} />
            <Row label="Nodes" value={String(project.nodes.length)} />
            <Row label="Edges" value={String(project.edges.length)} />
            <Row label="Creases" value={String(project.creases.length)} />
          </>
        )}
      </div>
    </section>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="control-row">
      <span className="control-row__label">{label}</span>
      <span className="control-row__value">{value}</span>
    </div>
  );
}
