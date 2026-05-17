import { CircleDot, MousePointer2, Plus, Waypoints, ZoomIn } from 'lucide-react';
import { IconButton } from '../ui/IconButton';
import { SegmentedControl } from '../ui/SegmentedControl';
import { formatNumber, paperToSvg, type PlotRect } from '../../lib/geometry';
import { useWorkspaceStore } from '../../store/workspaceStore';
import type { ToolMode } from '../../lib/sampleProject';

const VIEWBOX = 720;
const PAPER_RECT: PlotRect = { x: 66, y: 54, width: 588, height: 588 };

const toolOptions: Array<{ value: ToolMode; label: string; title: string }> = [
  { value: 'select', label: 'Select', title: 'Select and move parts' },
  { value: 'node', label: 'Node', title: 'Add nodes' },
  { value: 'edge', label: 'Edge', title: 'Add edges' },
];

export function DesignPanel() {
  const project = useWorkspaceStore((state) => state.project);
  const selection = useWorkspaceStore((state) => state.selection);
  const select = useWorkspaceStore((state) => state.select);
  const toolMode = useWorkspaceStore((state) => state.toolMode);
  const setToolMode = useWorkspaceStore((state) => state.setToolMode);

  const findNode = (id: number) => project.nodes.find((node) => node.id === id);

  return (
    <section className="panel-shell design-panel">
      <div className="panel-toolbar">
        <div className="panel-toolbar__group">
          <span className="panel-title">Design</span>
          <SegmentedControl
            aria-label="Tool"
            options={toolOptions}
            value={toolMode}
            onChange={setToolMode}
          />
        </div>
        <div className="panel-toolbar__group">
          <IconButton size="sm" title="Fit" tooltipSide="bottom">
            <ZoomIn size={14} />
          </IconButton>
          <IconButton size="sm" title="Select" tooltipSide="bottom" isActive={toolMode === 'select'}>
            <MousePointer2 size={14} />
          </IconButton>
        </div>
      </div>
      <div className="panel-body design-panel__body">
        <svg
          className="design-canvas"
          viewBox={`0 0 ${VIEWBOX} ${VIEWBOX}`}
          role="img"
          aria-label="Tree design canvas"
          onClick={() => select({ kind: 'tree' })}
        >
          <defs>
            <pattern id="paper-grid" width="58.8" height="58.8" patternUnits="userSpaceOnUse">
              <path d="M 58.8 0 L 0 0 0 58.8" fill="none" stroke="rgba(16,20,23,0.14)" strokeWidth="1" />
            </pattern>
          </defs>
          <rect className="paper-shadow" x="56" y="44" width="608" height="608" rx="6" />
          <rect
            className="paper"
            x={PAPER_RECT.x}
            y={PAPER_RECT.y}
            width={PAPER_RECT.width}
            height={PAPER_RECT.height}
          />
          <rect
            x={PAPER_RECT.x}
            y={PAPER_RECT.y}
            width={PAPER_RECT.width}
            height={PAPER_RECT.height}
            fill="url(#paper-grid)"
          />
          {project.paths.map((path) => {
            const a = findNode(path.nodes[0]);
            const b = findNode(path.nodes[1]);
            if (!a || !b) return null;
            const p1 = paperToSvg(a.loc, PAPER_RECT);
            const p2 = paperToSvg(b.loc, PAPER_RECT);
            const className = path.isActive
              ? 'tree-path tree-path--active'
              : path.isFeasible
                ? 'tree-path tree-path--feasible'
                : 'tree-path tree-path--bad';
            return (
              <line
                key={path.id}
                className={className}
                x1={p1.x}
                y1={p1.y}
                x2={p2.x}
                y2={p2.y}
              />
            );
          })}
          {project.edges.map((edge) => {
            const a = findNode(edge.nodes[0]);
            const b = findNode(edge.nodes[1]);
            if (!a || !b) return null;
            const p1 = paperToSvg(a.loc, PAPER_RECT);
            const p2 = paperToSvg(b.loc, PAPER_RECT);
            const active = selection.kind === 'edge' && selection.id === edge.id;
            return (
              <g key={edge.id} onClick={(event) => {
                event.stopPropagation();
                select({ kind: 'edge', id: edge.id });
              }}>
                <line
                  className={active ? 'tree-edge tree-edge--selected' : 'tree-edge'}
                  x1={p1.x}
                  y1={p1.y}
                  x2={p2.x}
                  y2={p2.y}
                />
                <text className="edge-label" x={(p1.x + p2.x) / 2 + 8} y={(p1.y + p2.y) / 2 - 8}>
                  {formatNumber(edge.length, 2)}
                </text>
              </g>
            );
          })}
          {project.nodes.map((node) => {
            const point = paperToSvg(node.loc, PAPER_RECT);
            const active = selection.kind === 'node' && selection.id === node.id;
            const incidentEdge = project.edges.find((edge) => edge.nodes.includes(node.id));
            const radius = node.isLeaf
              ? Math.max(22, (incidentEdge?.length ?? 1) * project.scale * PAPER_RECT.width)
              : 0;
            return (
              <g key={node.id} onClick={(event) => {
                event.stopPropagation();
                select({ kind: 'node', id: node.id });
              }}>
                {node.isLeaf && (
                  <circle
                    className="leaf-radius"
                    cx={point.x}
                    cy={point.y}
                    r={Math.max(22, radius)}
                  />
                )}
                <circle
                  className={active ? 'tree-node tree-node--selected' : 'tree-node'}
                  data-leaf={node.isLeaf || undefined}
                  cx={point.x}
                  cy={point.y}
                  r={node.isLeaf ? 7 : 8}
                />
                <text className="node-label" x={point.x + 11} y={point.y + 4}>
                  {node.label}
                </text>
              </g>
            );
          })}
        </svg>
        <div className="design-legend">
          <span><CircleDot size={13} /> Terminal</span>
          <span><Waypoints size={13} /> Active path</span>
          <span><Plus size={13} /> Scale {formatNumber(project.scale, 3)}</span>
        </div>
      </div>
    </section>
  );
}
