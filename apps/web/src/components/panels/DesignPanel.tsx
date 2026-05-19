import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type PointerEvent,
  type ReactNode,
} from 'react';
import { TransformComponent, TransformWrapper, type ReactZoomPanPinchRef } from 'react-zoom-pan-pinch';
import {
  Axis3d,
  Circle,
  CircleDot,
  Layers,
  Maximize2,
  Plus,
  Tag,
  Waypoints,
  ZoomIn,
  ZoomOut,
} from 'lucide-react';
import { formatNumber, paperToSvg, type Point } from '../../lib/geometry';
import {
  DEFAULT_DESIGN_VIEW_LAYERS,
  DESIGN_PAPER_RECT,
  DESIGN_PAPER_SHADOW_RECT,
  clientPointToPaper,
  getDesignWorldRect,
  leafCircleRadius,
  setDesignLayerVisibility,
  type DesignViewLayerKey,
  type DesignViewLayers,
} from '../../lib/designViewport';
import {
  isEdgeSelected,
  isNodeSelected,
  isPathSelected,
  toggleEdgeSelection,
  toggleNodeSelection,
} from '../../lib/selection';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { IconButton } from '../ui/IconButton';

const ZOOM_PRESETS = [25, 50, 100, 200, 400];

const LAYER_OPTIONS: { key: DesignViewLayerKey; label: string; icon: ReactNode }[] = [
  { key: 'paths', label: 'Paths', icon: <Waypoints size={13} /> },
  { key: 'leafCircles', label: 'Circles', icon: <Circle size={13} /> },
  { key: 'labels', label: 'Labels', icon: <Tag size={13} /> },
  { key: 'symmetry', label: 'Symmetry', icon: <Axis3d size={13} /> },
];

function isInteractiveTarget(target: EventTarget | null): boolean {
  return target instanceof Element && Boolean(target.closest('button, input, textarea, select, [role="menu"]'));
}

function formatZoom(scale: number): string {
  return `${Math.round(scale * 100)}%`;
}

function viewBox(rect: { x: number; y: number; width: number; height: number }): string {
  return `${rect.x} ${rect.y} ${rect.width} ${rect.height}`;
}

interface DesignViewportToolbarProps {
  zoomPercent: number;
  layers: DesignViewLayers;
  onLayerChange: (layer: DesignViewLayerKey, visible: boolean) => void;
  zoomIn: () => void;
  zoomOut: () => void;
  fitToView: () => void;
  setActualSize: () => void;
  setZoomLevel: (scale: number) => void;
}

function DesignViewportToolbar({
  zoomPercent,
  layers,
  onLayerChange,
  zoomIn,
  zoomOut,
  fitToView,
  setActualSize,
  setZoomLevel,
}: DesignViewportToolbarProps) {
  const [zoomMenuOpen, setZoomMenuOpen] = useState(false);
  const [layersOpen, setLayersOpen] = useState(false);
  const zoomMenuRef = useRef<HTMLDivElement | null>(null);
  const layersMenuRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!zoomMenuOpen && !layersOpen) return;
    const onPointerDown = (event: MouseEvent) => {
      const target = event.target as Node;
      if (zoomMenuRef.current?.contains(target) || layersMenuRef.current?.contains(target)) {
        return;
      }
      setZoomMenuOpen(false);
      setLayersOpen(false);
    };
    document.addEventListener('mousedown', onPointerDown);
    return () => document.removeEventListener('mousedown', onPointerDown);
  }, [layersOpen, zoomMenuOpen]);

  return (
    <div className="design-view-toolbar" aria-label="Design viewport controls">
      <IconButton size="sm" variant="toolbar" title="Zoom Out" onClick={zoomOut}>
        <ZoomOut size={14} />
      </IconButton>
      <div className="design-view-toolbar__menu-anchor" ref={zoomMenuRef}>
        <button
          type="button"
          className="design-view-toolbar__zoom-button"
          aria-haspopup="menu"
          aria-expanded={zoomMenuOpen}
          onClick={() => setZoomMenuOpen((open) => !open)}
        >
          {zoomPercent}%
        </button>
        {zoomMenuOpen && (
          <div className="design-view-toolbar__dropdown" role="menu">
            {ZOOM_PRESETS.map((preset) => (
              <button
                key={preset}
                type="button"
                className="design-view-toolbar__dropdown-item"
                onClick={() => {
                  setZoomLevel(preset / 100);
                  setZoomMenuOpen(false);
                }}
              >
                {preset}%
              </button>
            ))}
          </div>
        )}
      </div>
      <IconButton size="sm" variant="toolbar" title="Zoom In" onClick={zoomIn}>
        <ZoomIn size={14} />
      </IconButton>
      <span className="design-view-toolbar__separator" />
      <IconButton size="sm" variant="toolbar" title="Fit" onClick={fitToView}>
        <Maximize2 size={14} />
      </IconButton>
      <button type="button" className="design-view-toolbar__actual" onClick={setActualSize}>
        1:1
      </button>
      <span className="design-view-toolbar__separator" />
      <div className="design-view-toolbar__menu-anchor" ref={layersMenuRef}>
        <IconButton
          size="sm"
          variant="toolbar"
          title="Layers"
          isActive={layersOpen}
          onClick={() => setLayersOpen((open) => !open)}
        >
          <Layers size={14} />
        </IconButton>
        {layersOpen && (
          <div className="design-layer-menu" role="menu">
            {LAYER_OPTIONS.map((option) => (
              <label key={option.key} className="design-layer-option">
                <input
                  type="checkbox"
                  checked={layers[option.key]}
                  onChange={(event) => onLayerChange(option.key, event.target.checked)}
                />
                <span className="design-layer-option__icon">{option.icon}</span>
                <span>{option.label}</span>
              </label>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export function DesignPanel() {
  const svgRef = useRef<SVGSVGElement | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const transformRef = useRef<ReactZoomPanPinchRef | null>(null);
  const [dragging, setDragging] = useState<{
    id: number;
    start: Point;
    loc: Point;
    moved: boolean;
  } | null>(null);
  const [zoomPercent, setZoomPercent] = useState(100);
  const [layers, setLayers] = useState<DesignViewLayers>(DEFAULT_DESIGN_VIEW_LAYERS);
  const [spacePressed, setSpacePressed] = useState(false);
  const [hoverPoint, setHoverPoint] = useState<Point | null>(null);
  const project = useWorkspaceStore((state) => state.project);
  const selection = useWorkspaceStore((state) => state.selection);
  const select = useWorkspaceStore((state) => state.select);
  const addNodeAt = useWorkspaceStore((state) => state.addNodeAt);
  const moveNode = useWorkspaceStore((state) => state.moveNode);
  const projectLoadId = useWorkspaceStore((state) => state.projectLoadId);

  const nodeLocations = useMemo(() => {
    if (!dragging) return undefined;
    return new Map([[dragging.id, dragging.loc]]);
  }, [dragging]);
  const worldRect = useMemo(
    () => getDesignWorldRect(project, layers, { nodeLocations }),
    [layers, nodeLocations, project]
  );

  const findNode = (id: number) => project.nodes.find((node) => node.id === id);
  const displayLoc = (id: number, loc: Point) => (dragging?.id === id ? dragging.loc : loc);

  const symmetryLine = useMemo(() => {
    const center = paperToSvg(project.paper.symLoc, DESIGN_PAPER_RECT);
    const angle = (project.paper.symAngle * Math.PI) / 180;
    const span = Math.hypot(worldRect.width, worldRect.height);
    return {
      x1: center.x - Math.cos(angle) * span,
      y1: center.y + Math.sin(angle) * span,
      x2: center.x + Math.cos(angle) * span,
      y2: center.y - Math.sin(angle) * span,
    };
  }, [project.paper.symAngle, project.paper.symLoc, worldRect]);

  const eventToPaper = useCallback(
    (event: PointerEvent): Point => {
      const svg = svgRef.current;
      if (!svg) return { x: 0, y: 0 };
      return clientPointToPaper(
        { x: event.clientX, y: event.clientY },
        svg.getBoundingClientRect(),
        worldRect
      );
    },
    [worldRect]
  );

  const computeFitScale = useCallback(() => {
    const container = containerRef.current;
    if (!container) return 1;
    const padding = 96;
    const width = Math.max(1, container.clientWidth - padding);
    const height = Math.max(1, container.clientHeight - padding);
    return Math.max(0.05, Math.min(width / worldRect.width, height / worldRect.height, 1));
  }, [worldRect]);

  const fitToView = useCallback(
    (animationTime = 180) => {
      transformRef.current?.centerView(computeFitScale(), animationTime);
    },
    [computeFitScale]
  );

  const setActualSize = useCallback(() => {
    transformRef.current?.centerView(1, 160);
  }, []);

  const setZoomLevel = useCallback((scale: number) => {
    transformRef.current?.centerView(scale, 160);
  }, []);

  const lastFittedProjectLoadIdRef = useRef<number | null>(null);
  const fitLoadedProject = useCallback(
    (animationTime = 0) => {
      if (lastFittedProjectLoadIdRef.current === projectLoadId) return true;
      const container = containerRef.current;
      if (!container || !transformRef.current || container.clientWidth <= 0 || container.clientHeight <= 0) {
        return false;
      }
      transformRef.current.centerView(computeFitScale(), animationTime);
      lastFittedProjectLoadIdRef.current = projectLoadId;
      return true;
    },
    [computeFitScale, projectLoadId]
  );

  const fitLoadedProjectRef = useRef(fitLoadedProject);
  useEffect(() => {
    fitLoadedProjectRef.current = fitLoadedProject;
  }, [fitLoadedProject]);

  useEffect(() => {
    const container = containerRef.current;
    let frame = requestAnimationFrame(() => fitLoadedProjectRef.current(0));
    const observer =
      typeof ResizeObserver === 'undefined' || !container
        ? null
        : new ResizeObserver(() => {
            if (lastFittedProjectLoadIdRef.current !== projectLoadId) {
              cancelAnimationFrame(frame);
              frame = requestAnimationFrame(() => fitLoadedProjectRef.current(0));
            }
          });

    if (observer && container) {
      observer.observe(container);
    }
    return () => {
      cancelAnimationFrame(frame);
      observer?.disconnect();
    };
  }, [projectLoadId]);

  const setLayer = useCallback((layer: DesignViewLayerKey, visible: boolean) => {
    setLayers((current) => setDesignLayerVisibility(current, layer, visible));
  }, []);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return undefined;

    const onKeyDown = (event: KeyboardEvent) => {
      const interactive = isInteractiveTarget(event.target);
      if (event.key === ' ' && !interactive) {
        event.preventDefault();
        setSpacePressed(true);
        return;
      }

      if (interactive || (!event.metaKey && !event.ctrlKey)) return;

      switch (event.key) {
        case '=':
        case '+':
          event.preventDefault();
          transformRef.current?.zoomIn(0.35, 120);
          break;
        case '-':
        case '_':
          event.preventDefault();
          transformRef.current?.zoomOut(0.35, 120);
          break;
        case '0':
          event.preventDefault();
          fitToView();
          break;
        case '1':
          event.preventDefault();
          setActualSize();
          break;
      }
    };

    const onKeyUp = (event: KeyboardEvent) => {
      if (event.key === ' ') setSpacePressed(false);
    };
    const clearSpace = () => setSpacePressed(false);

    container.addEventListener('keydown', onKeyDown);
    container.addEventListener('keyup', onKeyUp);
    window.addEventListener('keyup', onKeyUp);
    window.addEventListener('blur', clearSpace);
    return () => {
      container.removeEventListener('keydown', onKeyDown);
      container.removeEventListener('keyup', onKeyUp);
      window.removeEventListener('keyup', onKeyUp);
      window.removeEventListener('blur', clearSpace);
    };
  }, [fitToView, setActualSize]);

  const onPaperPointerDown = (event: PointerEvent<SVGRectElement>) => {
    if (event.button !== 0 || spacePressed) return;
    const connectTo = selection.kind === 'node' ? selection.id : undefined;
    void addNodeAt(eventToPaper(event), connectTo);
  };

  const onNodePointerDown = (event: PointerEvent<SVGCircleElement>, nodeId: number) => {
    if (event.button !== 0 || spacePressed) return;
    event.stopPropagation();
    if (event.shiftKey || event.metaKey || event.ctrlKey) {
      select(toggleNodeSelection(selection, nodeId));
      return;
    }
    select({ kind: 'node', id: nodeId });
    const node = findNode(nodeId);
    if (!node) return;
    event.currentTarget.setPointerCapture(event.pointerId);
    setDragging({ id: nodeId, start: node.loc, loc: node.loc, moved: false });
  };

  const onNodePointerMove = (event: PointerEvent<SVGCircleElement>, nodeId: number) => {
    if (dragging?.id !== nodeId) return;
    event.stopPropagation();
    const loc = eventToPaper(event);
    setHoverPoint(loc);
    const dx = loc.x - dragging.start.x;
    const dy = loc.y - dragging.start.y;
    setDragging({
      id: nodeId,
      start: dragging.start,
      loc,
      moved: dragging.moved || Math.hypot(dx, dy) > 0.003,
    });
  };

  const finishDrag = (event: PointerEvent<SVGCircleElement>, nodeId: number) => {
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

  const onCanvasPointerMove = (event: PointerEvent<SVGSVGElement>) => {
    setHoverPoint(eventToPaper(event));
  };

  return (
    <section className="panel-shell design-panel">
      <div
        ref={containerRef}
        className="panel-body design-panel__body"
        data-space-pan={spacePressed || undefined}
        tabIndex={-1}
        onPointerDown={(event) => {
          if (!isInteractiveTarget(event.target)) containerRef.current?.focus();
        }}
      >
        <TransformWrapper
          ref={transformRef}
          initialScale={1}
          minScale={0.05}
          maxScale={30}
          centerOnInit
          limitToBounds={false}
          wheel={{ step: 0.5, wheelDisabled: true }}
          panning={{
            velocityDisabled: true,
            wheelPanning: true,
            allowMiddleClickPan: true,
            allowLeftClickPan: spacePressed,
          }}
          pinch={{ step: 0.5 }}
          doubleClick={{ disabled: true }}
          onInit={(ref) => {
            transformRef.current = ref;
            requestAnimationFrame(() => fitLoadedProjectRef.current(0));
          }}
          onTransformed={(_ref, state) => setZoomPercent(Math.round(state.scale * 100))}
        >
          <TransformComponent
            wrapperStyle={{ width: '100%', height: '100%' }}
            contentStyle={{ width: 'fit-content', height: 'fit-content' }}
          >
            <svg
              ref={svgRef}
              className="design-canvas"
              viewBox={viewBox(worldRect)}
              width={worldRect.width}
              height={worldRect.height}
              style={{ width: worldRect.width, height: worldRect.height }}
              role="img"
              aria-label="Tree design canvas"
              onPointerMove={onCanvasPointerMove}
              onPointerLeave={() => setHoverPoint(null)}
            >
              <rect
                className="paper-shadow"
                x={DESIGN_PAPER_SHADOW_RECT.x}
                y={DESIGN_PAPER_SHADOW_RECT.y}
                width={DESIGN_PAPER_SHADOW_RECT.width}
                height={DESIGN_PAPER_SHADOW_RECT.height}
                rx="6"
              />
              <rect
                className="paper"
                x={DESIGN_PAPER_RECT.x}
                y={DESIGN_PAPER_RECT.y}
                width={DESIGN_PAPER_RECT.width}
                height={DESIGN_PAPER_RECT.height}
              />
              <rect
                className="paper-hit-area"
                x={DESIGN_PAPER_RECT.x}
                y={DESIGN_PAPER_RECT.y}
                width={DESIGN_PAPER_RECT.width}
                height={DESIGN_PAPER_RECT.height}
                onPointerDown={onPaperPointerDown}
              />
              {project.hasSymmetry && layers.symmetry && (
                <line
                  className="symmetry-line"
                  x1={symmetryLine.x1}
                  y1={symmetryLine.y1}
                  x2={symmetryLine.x2}
                  y2={symmetryLine.y2}
                />
              )}
              {layers.paths &&
                project.paths.map((path) => {
                  const a = findNode(path.nodes[0]);
                  const b = findNode(path.nodes[1]);
                  if (!a || !b) return null;
                  const p1 = paperToSvg(displayLoc(a.id, a.loc), DESIGN_PAPER_RECT);
                  const p2 = paperToSvg(displayLoc(b.id, b.loc), DESIGN_PAPER_RECT);
                  const className = !path.isLeaf
                    ? 'tree-path tree-path--internal'
                    : path.isActive
                      ? 'tree-path tree-path--active'
                      : path.isFeasible
                        ? 'tree-path tree-path--feasible'
                        : 'tree-path tree-path--bad';
                  const conditioned = path.isConditioned ? 'tree-path--conditioned' : '';
                  const active = isPathSelected(selection, path.id);
                  return (
                    <line
                      key={path.id}
                      className={`${className} ${conditioned} ${active ? 'tree-path--selected' : ''}`}
                      x1={p1.x}
                      y1={p1.y}
                      x2={p2.x}
                      y2={p2.y}
                      onPointerDown={(event) => {
                        if (spacePressed) return;
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
                const p1 = paperToSvg(displayLoc(a.id, a.loc), DESIGN_PAPER_RECT);
                const p2 = paperToSvg(displayLoc(b.id, b.loc), DESIGN_PAPER_RECT);
                const active = isEdgeSelected(selection, edge.id);
                return (
                  <g
                    key={edge.id}
                    onPointerDown={(event) => {
                      if (spacePressed) return;
                      event.stopPropagation();
                      select(
                        event.shiftKey || event.metaKey || event.ctrlKey
                          ? toggleEdgeSelection(selection, edge.id)
                          : { kind: 'edge', id: edge.id }
                      );
                    }}
                  >
                    <line
                      className={[
                        'tree-edge',
                        edge.isConditioned ? 'tree-edge--conditioned' : '',
                        active ? 'tree-edge--selected' : '',
                      ].join(' ')}
                      x1={p1.x}
                      y1={p1.y}
                      x2={p2.x}
                      y2={p2.y}
                    />
                    {layers.labels && (
                      <text className="edge-label" x={(p1.x + p2.x) / 2 + 8} y={(p1.y + p2.y) / 2 - 8}>
                        {formatNumber(edge.length, 2)}
                      </text>
                    )}
                  </g>
                );
              })}
              {project.nodes.map((node) => {
                const point = paperToSvg(displayLoc(node.id, node.loc), DESIGN_PAPER_RECT);
                const active = isNodeSelected(selection, node.id);
                const radius = node.isLeaf ? leafCircleRadius(project, node.id) : 0;
                return (
                  <g key={node.id}>
                    {node.isLeaf && layers.leafCircles && (
                      <circle className="leaf-radius" cx={point.x} cy={point.y} r={radius} />
                    )}
                    <circle
                      className={[
                        'tree-node',
                        node.isConditioned ? 'tree-node--conditioned' : '',
                        active ? 'tree-node--selected' : '',
                      ].join(' ')}
                      data-leaf={node.isLeaf || undefined}
                      cx={point.x}
                      cy={point.y}
                      r={node.isLeaf ? 7 : 8}
                      onPointerDown={(event) => onNodePointerDown(event, node.id)}
                      onPointerMove={(event) => onNodePointerMove(event, node.id)}
                      onPointerUp={(event) => finishDrag(event, node.id)}
                      onPointerCancel={(event) => finishDrag(event, node.id)}
                    />
                    {layers.labels && (
                      <text className="node-label" x={point.x + 11} y={point.y + 4}>
                        {node.label}
                      </text>
                    )}
                  </g>
                );
              })}
            </svg>
          </TransformComponent>
        </TransformWrapper>
        <DesignViewportToolbar
          zoomPercent={zoomPercent}
          layers={layers}
          onLayerChange={setLayer}
          zoomIn={() => transformRef.current?.zoomIn(0.35, 120)}
          zoomOut={() => transformRef.current?.zoomOut(0.35, 120)}
          fitToView={() => fitToView()}
          setActualSize={setActualSize}
          setZoomLevel={setZoomLevel}
        />
        <div className="design-status-readout">
          <span>{formatZoom(zoomPercent / 100)}</span>
          {hoverPoint && (
            <span>
              {formatNumber(hoverPoint.x, 3)}, {formatNumber(hoverPoint.y, 3)}
            </span>
          )}
        </div>
        <div className="design-legend">
          <span><CircleDot size={13} /> Terminal</span>
          <span><Waypoints size={13} /> Active path</span>
          <span><Plus size={13} /> Scale {formatNumber(project.scale, 3)}</span>
        </div>
      </div>
    </section>
  );
}
