import { useEffect, useState } from 'react';
import { Activity, Circle, GitBranch, MousePointer2, Square, Waypoints } from 'lucide-react';
import { formatNumber } from '../../lib/geometry';
import { selectedNodeIds, selectionSummary } from '../../lib/selection';
import { useWorkspaceStore } from '../../store/workspaceStore';

export function InspectorPanel() {
  const project = useWorkspaceStore((state) => state.project);
  const selection = useWorkspaceStore((state) => state.selection);
  const moveNode = useWorkspaceStore((state) => state.moveNode);
  const updateNodeLabel = useWorkspaceStore((state) => state.updateNodeLabel);
  const updateEdge = useWorkspaceStore((state) => state.updateEdge);
  const selectPathBetweenSelectedNodes = useWorkspaceStore(
    (state) => state.selectPathBetweenSelectedNodes
  );

  const selectedNode =
    selection.kind === 'node' ? project.nodes.find((node) => node.id === selection.id) : null;
  const selectedEdge =
    selection.kind === 'edge' ? project.edges.find((edge) => edge.id === selection.id) : null;
  const selectedCrease =
    selection.kind === 'crease' ? project.creases.find((crease) => crease.id === selection.id) : null;
  const selectedFacet =
    selection.kind === 'facet' ? project.facets.find((facet) => facet.id === selection.id) : null;
  const selectedPath =
    selection.kind === 'path' ? project.paths.find((path) => path.id === selection.id) : null;
  const selectedNodes = selectedNodeIds(selection);

  return (
    <section className="panel-shell inspector-panel">
      <div className="panel-toolbar">
        <span className="panel-title">Inspector</span>
      </div>
      <div className="panel-body">
        {selectedNode && (
          <>
            <div className="inspector-heading"><Circle size={15} /> Node {selectedNode.id}</div>
            <EditableRow
              label="Label"
              value={selectedNode.label}
              onCommit={(label) => void updateNodeLabel(selectedNode.id, label)}
            />
            <NumberRow
              label="X"
              value={selectedNode.loc.x}
              min={0}
              max={1}
              step={0.01}
              onCommit={(x) => void moveNode(selectedNode.id, { ...selectedNode.loc, x })}
            />
            <NumberRow
              label="Y"
              value={selectedNode.loc.y}
              min={0}
              max={1}
              step={0.01}
              onCommit={(y) => void moveNode(selectedNode.id, { ...selectedNode.loc, y })}
            />
            <Row label="Leaf" value={selectedNode.isLeaf ? 'Yes' : 'No'} />
            <Row label="Pinned" value={selectedNode.isPinned ? 'Yes' : 'No'} />
          </>
        )}
        {selectedEdge && (
          <>
            <div className="inspector-heading"><GitBranch size={15} /> Edge {selectedEdge.id}</div>
            <EditableRow
              label="Label"
              value={selectedEdge.label}
              onCommit={(label) => void updateEdge(selectedEdge.id, { label })}
            />
            <Row label="Nodes" value={selectedEdge.nodes.join(' -> ')} />
            <NumberRow
              label="Length"
              value={selectedEdge.length}
              min={0.001}
              step={0.05}
              onCommit={(length) => void updateEdge(selectedEdge.id, { length })}
            />
            <Row label="Strain" value={formatNumber(selectedEdge.strain)} />
            <NumberRow
              label="Stiffness"
              value={selectedEdge.stiffness}
              min={0.001}
              step={0.1}
              onCommit={(stiffness) => void updateEdge(selectedEdge.id, { stiffness })}
            />
          </>
        )}
        {selectedCrease && (
          <>
            <div className="inspector-heading"><Activity size={15} /> Crease {selectedCrease.id}</div>
            <Row label="Fold" value={selectedCrease.fold} />
            <Row label="Kind" value={selectedCrease.kind} />
          </>
        )}
        {selectedPath && (
          <>
            <div className="inspector-heading"><Waypoints size={15} /> Path {selectedPath.id}</div>
            <Row label="Nodes" value={selectedPath.nodes.join(' -> ')} />
            <Row label="Active" value={selectedPath.isActive ? 'Yes' : 'No'} />
            <Row label="Feasible" value={selectedPath.isFeasible ? 'Yes' : 'No'} />
            <Row label="Border" value={selectedPath.isBorder ? 'Yes' : 'No'} />
          </>
        )}
        {selectedFacet && (
          <>
            <div className="inspector-heading"><Activity size={15} /> Facet {selectedFacet.id}</div>
            <Row label="Vertices" value={String(selectedFacet.vertices.length)} />
            <Row label="Color" value={selectedFacet.color} />
          </>
        )}
        {selection.kind === 'multi' && (
          <>
            <div className="inspector-heading"><MousePointer2 size={15} /> Selection</div>
            <Row label="Parts" value={selectionSummary(selection)} />
            {selectedNodes.length === 2 && (
              <button
                className="control-row control-row--button"
                type="button"
                onClick={selectPathBetweenSelectedNodes}
              >
                <span className="control-row__label">Path</span>
                <span className="control-row__value">Select between nodes</span>
              </button>
            )}
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

function EditableRow({
  label,
  value,
  onCommit,
}: {
  label: string;
  value: string;
  onCommit: (value: string) => void;
}) {
  const [draft, setDraft] = useState(value);

  useEffect(() => {
    setDraft(value);
  }, [value]);

  const commit = () => {
    const next = draft.trim();
    if (next && next !== value) onCommit(next);
    else setDraft(value);
  };

  return (
    <label className="control-row">
      <span className="control-row__label">{label}</span>
      <input
        className="control-row__input"
        value={draft}
        onChange={(event) => setDraft(event.target.value)}
        onBlur={commit}
        onKeyDown={(event) => {
          if (event.key === 'Enter') event.currentTarget.blur();
          if (event.key === 'Escape') {
            setDraft(value);
            event.currentTarget.blur();
          }
        }}
      />
    </label>
  );
}

function NumberRow({
  label,
  value,
  min,
  max,
  step,
  onCommit,
}: {
  label: string;
  value: number;
  min?: number;
  max?: number;
  step: number;
  onCommit: (value: number) => void;
}) {
  const [draft, setDraft] = useState(formatNumber(value, 4));

  useEffect(() => {
    setDraft(formatNumber(value, 4));
  }, [value]);

  const commit = () => {
    const parsed = Number.parseFloat(draft);
    if (!Number.isFinite(parsed)) {
      setDraft(formatNumber(value, 4));
      return;
    }
    const lowerBounded = min === undefined ? parsed : Math.max(min, parsed);
    const next = max === undefined ? lowerBounded : Math.min(max, lowerBounded);
    if (Math.abs(next - value) > 0.000_001) onCommit(next);
    setDraft(formatNumber(next, 4));
  };

  return (
    <label className="control-row">
      <span className="control-row__label">{label}</span>
      <input
        className="control-row__input"
        type="number"
        min={min}
        max={max}
        step={step}
        value={draft}
        onChange={(event) => setDraft(event.target.value)}
        onBlur={commit}
        onKeyDown={(event) => {
          if (event.key === 'Enter') event.currentTarget.blur();
          if (event.key === 'Escape') {
            setDraft(formatNumber(value, 4));
            event.currentTarget.blur();
          }
        }}
      />
    </label>
  );
}
