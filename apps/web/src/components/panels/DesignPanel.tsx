import { useRef, useState, type PointerEvent } from 'react';
import { CircleDot, MousePointer2, Plus, Waypoints, ZoomIn } from 'lucide-react';
import { IconButton } from '../ui/IconButton';
import { SegmentedControl } from '../ui/SegmentedControl';
import {
  clampPaperPoint,
  formatNumber,
  paperToSvg,
  svgToPaper,
  type PlotRect,
  type Point,
} from '../../lib/geometry';
import {
  isEdgeSelected,
  isNodeSelected,
  isPathSelected,
  toggleEdgeSelection,
  toggleNodeSelection,
} from '../../lib/selection';
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
  const svgRef = useRef<SVGSVGElement | null>(null);
  const [dragging, setDragging] = useState<{
    id: number;
    start: Point;
    loc: Point;
    moved: boolean;
  } | null>(null);
  const project = useWorkspaceStore((state) => state.project);
  const selection = useWorkspaceStore((state) => state.selection);
  const select = useWorkspaceStore((state) => state.select);
  const toolMode = useWorkspaceStore((state) => state.toolMode);
  const setToolMode = useWorkspaceStore((state) => state.setToolMode);
  const addNodeAt = useWorkspaceStore((state) => state.addNodeAt);
  const addEdge = useWorkspaceStore((state) => state.addEdge);
  const moveNode = useWorkspaceStore((state) => state.moveNode);

  const findNode = (id: number) => project.nodes.find((node) => node.id === id);
  const displayLoc = (id: number, loc: Point) => (dragging?.id === id ? dragging.loc : loc);

  const eventToPaper = (event: PointerEvent): Point => {
    const svg = svgRef.current;
    if (!svg) return { x: 0, y: 0 };
    const bounds = svg.getBoundingClientRect();
    const point = {
      x: ((event.clientX - bounds.left) / bounds.width) * VIEWBOX,
      y: ((event.clientY - bounds.top) / bounds.height) * VIEWBOX,
    };
    return clampPaperPoint(svgToPaper(point, PAPER_RECT));
  };

  const onPaperPointerDown = (event: PointerEvent<SVGRectElement>) => {
    if (event.button !== 0) return;
    const connectTo = selection.kind === 'node' ? selection.id : undefined;
    void addNodeAt(eventToPaper(event), connectTo);
  };

  const onNodePointerDown = (event: PointerEvent<SVGGElement>, nodeId: number) => {
    if (event.button !== 0) return;
    event.stopPropagation();
    if (event.shiftKey || event.metaKey || event.ctrlKey) {
      select(toggleNodeSelection(selection, nodeId));
      return;
    }
    if (toolMode === 'edge') {
      if (selection.kind === 'node' && selection.id !== nodeId) {
        void addEdge(selection.id, nodeId);
      } else {
        select({ kind: 'node', id: nodeId });
      }
      return;
    }
    select({ kind: 'node', id: nodeId });
    if (toolMode === 'select') {
      const node = findNode(nodeId);
      if (!node) return;
      event.currentTarget.setPointerCapture(event.pointerId);
      setDragging({ id: nodeId, start: node.loc, loc: node.loc, moved: false });
    }
  };

  const onNodePointerMove = (event: PointerEvent<SVGGElement>, nodeId: number) => {
    if (dragging?.id !== nodeId) return;
    event.stopPropagation();
    const loc = eventToPaper(event);
    const dx = loc.x - dragging.start.x;
    const dy = loc.y - dragging.start.y;
    setDragging({
      id: nodeId,
      start: dragging.start,
      loc,
      moved: dragging.moved || Math.hypot(dx, dy) > 0.003,
    });
  };

  const finishDrag = (event: PointerEvent<SVGGElement>, nodeId: number) => {
    if (dragging?.id !== nodeId) return;
    event.stopPropagation();
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    const loc = dragging.loc;
    const moved = dragging.moved;
    setDragging(null);
    if (moved) void moveNode(nodeId, loc);
  };

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
          ref={svgRef}
          className="design-canvas"
          data-tool={toolMode}
          viewBox={`0 0 ${VIEWBOX} ${VIEWBOX}`}
          role="img"
          aria-label="Tree design canvas"
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
          <rect
            className="paper-hit-area"
            x={PAPER_RECT.x}
            y={PAPER_RECT.y}
            width={PAPER_RECT.width}
            height={PAPER_RECT.height}
            onPointerDown={onPaperPointerDown}
          />
          {project.paths.map((path) => {
            const a = findNode(path.nodes[0]);
            const b = findNode(path.nodes[1]);
            if (!a || !b) return null;
            const p1 = paperToSvg(displayLoc(a.id, a.loc), PAPER_RECT);
            const p2 = paperToSvg(displayLoc(b.id, b.loc), PAPER_RECT);
            const className = path.isActive
              ? 'tree-path tree-path--active'
              : path.isFeasible
                ? 'tree-path tree-path--feasible'
                : 'tree-path tree-path--bad';
            const active = isPathSelected(selection, path.id);
            return (
              <line
                key={path.id}
                className={`${className} ${active ? 'tree-path--selected' : ''}`}
                x1={p1.x}
                y1={p1.y}
                x2={p2.x}
                y2={p2.y}
                onPointerDown={(event) => {
                  event.stopPropagation();
                  select({ kind: 'path', id: path.id });
                }}
              />
            );
          })}
          {project.edges.map((edge) => {
            const a = findNode(edge.nodes[0]);
            const b = findNode(edge.nodes[1]);
            if (!a || !b) return null;
            const p1 = paperToSvg(displayLoc(a.id, a.loc), PAPER_RECT);
            const p2 = paperToSvg(displayLoc(b.id, b.loc), PAPER_RECT);
            const active = isEdgeSelected(selection, edge.id);
            return (
              <g key={edge.id} onPointerDown={(event) => {
                event.stopPropagation();
                select(
                  event.shiftKey || event.metaKey || event.ctrlKey
                    ? toggleEdgeSelection(selection, edge.id)
                    : { kind: 'edge', id: edge.id }
                );
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
            const point = paperToSvg(displayLoc(node.id, node.loc), PAPER_RECT);
            const active = isNodeSelected(selection, node.id);
            const pendingEdge = toolMode === 'edge' && active;
            const incidentEdge = project.edges.find((edge) => edge.nodes.includes(node.id));
            const radius = node.isLeaf
              ? Math.max(22, (incidentEdge?.length ?? 1) * project.scale * PAPER_RECT.width)
              : 0;
            return (
              <g
                key={node.id}
                onPointerDown={(event) => onNodePointerDown(event, node.id)}
                onPointerMove={(event) => onNodePointerMove(event, node.id)}
                onPointerUp={(event) => finishDrag(event, node.id)}
                onPointerCancel={(event) => finishDrag(event, node.id)}
              >
                {node.isLeaf && (
                  <circle
                    className="leaf-radius"
                    cx={point.x}
                    cy={point.y}
                    r={Math.max(22, radius)}
                  />
                )}
                <circle
                  className={[
                    'tree-node',
                    active ? 'tree-node--selected' : '',
                    pendingEdge ? 'tree-node--pending-edge' : '',
                  ].join(' ')}
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
