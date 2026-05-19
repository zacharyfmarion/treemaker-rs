import {
  useCallback,
  useEffect,
  useLayoutEffect,
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
  FileQuestionMark,
  FileText,
  FolderOpen,
  Layers,
  Link2,
  Maximize2,
  Plus,
  ScanLine,
  Tag,
  Waypoints,
  ZoomIn,
  ZoomOut,
} from 'lucide-react';
import { handleMenuAction } from '../../commands/menuActions';
import { formatNumber, paperToSvg, type Point } from '../../lib/geometry';
import {
  DEFAULT_DESIGN_VIEW_LAYERS,
  DESIGN_PAPER_RECT,
  DESIGN_PAPER_SHADOW_RECT,
  type ViewportSize,
  clientPointToPaper,
  getCenteredDesignTransform,
  getDesignWorldRect,
  getViewportFitScale,
  leafCircleRadius,
  setDesignLayerVisibility,
  type DesignViewLayerKey,
  type DesignViewLayers,
} from '../../lib/designViewport';
import {
  isEdgeSelected,
  isNodeSelected,
  isPathSelected,
  selectedNodeIds,
  toggleEdgeSelection,
  toggleNodeSelection,
} from '../../lib/selection';
import { paperCenter } from '../../lib/symmetryPresets';
import {
  findMirrorNodeId,
  reflectPointAcrossSymmetryAxis,
  snapPointToSymmetryAxis,
  symmetryAxisForProject,
  symmetrySide,
  type SymmetryLeafPreview,
} from '../../lib/symmetryAuthoring';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { Button } from '../ui/Button';
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
  mirrorMode: boolean;
  hasSymmetry: boolean;
  onLayerChange: (layer: DesignViewLayerKey, visible: boolean) => void;
  onToggleMirror: () => void;
  onPairLeaves: () => void;
  zoomIn: () => void;
  zoomOut: () => void;
  fitToView: () => void;
  setActualSize: () => void;
  setZoomLevel: (scale: number) => void;
}

function DesignViewportToolbar({
  zoomPercent,
  layers,
  mirrorMode,
  hasSymmetry,
  onLayerChange,
  onToggleMirror,
  onPairLeaves,
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
      <IconButton
        size="sm"
        variant="toolbar"
        title={mirrorMode ? 'Mirror On' : 'Mirror'}
        isActive={mirrorMode}
        onClick={onToggleMirror}
      >
        <Axis3d size={14} />
      </IconButton>
      <IconButton
        size="sm"
        variant="toolbar"
        title="Pair Leaves"
        disabled={!hasSymmetry}
        onClick={onPairLeaves}
      >
        <Link2 size={14} />
      </IconButton>
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
  const engineReady = useWorkspaceStore((state) => state.engineReady);
  const documentMode = useWorkspaceStore((state) => state.documentMode);
  const importedCreasePattern = useWorkspaceStore((state) => state.importedCreasePattern);
  const selection = useWorkspaceStore((state) => state.selection);
  const toolMode = useWorkspaceStore((state) => state.toolMode);
  const symmetryAuthoringPairs = useWorkspaceStore((state) => state.symmetryAuthoringPairs);
  const select = useWorkspaceStore((state) => state.select);
  const addNodeAt = useWorkspaceStore((state) => state.addNodeAt);
  const addNodeWithSymmetry = useWorkspaceStore((state) => state.addNodeWithSymmetry);
  const moveNode = useWorkspaceStore((state) => state.moveNode);
  const moveNodeWithSymmetry = useWorkspaceStore((state) => state.moveNodeWithSymmetry);
  const setToolMode = useWorkspaceStore((state) => state.setToolMode);
  const setSymmetry = useWorkspaceStore((state) => state.setSymmetry);
  const previewSymmetryLeafPairs = useWorkspaceStore((state) => state.previewSymmetryLeafPairs);
  const applySymmetryLeafPairs = useWorkspaceStore((state) => state.applySymmetryLeafPairs);
  const projectLoadId = useWorkspaceStore((state) => state.projectLoadId);
  const designViewportFitRequestId = useWorkspaceStore(
    (state) => state.designViewportFitRequestId
  );
  const [symmetryPreview, setSymmetryPreview] = useState<SymmetryLeafPreview | null>(null);
  const mirrorMode = toolMode === 'symmetry';
  const symmetryAxis = useMemo(() => symmetryAxisForProject(project), [project]);
  const showEmptyState = engineReady && project.nodes.length === 0 && project.edges.length === 0;

  const nodeLocations = useMemo(() => {
    if (!dragging) return undefined;
    const locations = new Map([[dragging.id, dragging.loc]]);
    if (mirrorMode && project.hasSymmetry) {
      const pairedNode = findMirrorNodeId(project, symmetryAuthoringPairs, dragging.id);
      if (pairedNode) {
        locations.set(pairedNode, reflectPointAcrossSymmetryAxis(dragging.loc, symmetryAxis));
      }
    }
    return locations;
  }, [dragging, mirrorMode, project, symmetryAuthoringPairs, symmetryAxis]);
  const worldRect = useMemo(
    () => getDesignWorldRect(project, layers, { nodeLocations }),
    [layers, nodeLocations, project]
  );

  const findNode = (id: number) => project.nodes.find((node) => node.id === id);
  const displayLoc = (id: number, loc: Point) => nodeLocations?.get(id) ?? loc;

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

  const selectedLeafNodeIds = useMemo(
    () =>
      selectedNodeIds(selection).filter((id) =>
        project.nodes.some((node) => node.id === id && node.isLeaf)
      ),
    [project.nodes, selection]
  );

  const symmetryHoverPreview = useMemo(() => {
    if (!mirrorMode || !project.hasSymmetry || selection.kind !== 'node' || !hoverPoint) return null;
    const parent = project.nodes.find((node) => node.id === selection.id);
    if (!parent) return null;
    const snapped = snapPointToSymmetryAxis(hoverPoint, symmetryAxis);
    const parentSide = symmetrySide(parent.loc, symmetryAxis);
    const pairedParentId = findMirrorNodeId(project, symmetryAuthoringPairs, parent.id);
    const pairedParent = pairedParentId
      ? project.nodes.find((node) => node.id === pairedParentId)
      : null;
    const shouldMirror = !snapped.snapped && (parentSide === 0 || pairedParent);
    return {
      primary: { from: parent.loc, to: snapped.point },
      mirror:
        shouldMirror && (parentSide === 0 || pairedParent)
          ? {
              from: parentSide === 0 ? parent.loc : pairedParent!.loc,
              to: reflectPointAcrossSymmetryAxis(snapped.point, symmetryAxis),
            }
          : null,
      snapped: snapped.snapped,
      unresolved: !snapped.snapped && parentSide !== 0 && !pairedParent,
    };
  }, [hoverPoint, mirrorMode, project, selection, symmetryAuthoringPairs, symmetryAxis]);

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

  const getViewportSize = useCallback((): ViewportSize | null => {
    const viewport = containerRef.current;
    if (!viewport) return null;
    return {
      width: viewport.clientWidth || viewport.offsetWidth,
      height: viewport.clientHeight || viewport.offsetHeight,
    };
  }, []);

  const computeFitScale = useCallback(() => {
    const viewport = getViewportSize();
    if (!viewport) return 1;
    return getViewportFitScale(viewport, worldRect);
  }, [getViewportSize, worldRect]);

  const fitPaperToView = useCallback(
    (animationTime = 180) => {
      const viewport = getViewportSize();
      if (!viewport) return;
      const transform = getCenteredDesignTransform(viewport, worldRect, DESIGN_PAPER_RECT);
      transformRef.current?.setTransform(
        transform.positionX,
        transform.positionY,
        transform.scale,
        animationTime
      );
    },
    [getViewportSize, worldRect]
  );

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
  const lastHandledFitRequestRef = useRef(0);
  useLayoutEffect(() => {
    if (designViewportFitRequestId === 0) return undefined;
    if (lastHandledFitRequestRef.current === designViewportFitRequestId) return undefined;
    lastHandledFitRequestRef.current = designViewportFitRequestId;
    lastFittedProjectLoadIdRef.current = projectLoadId;

    fitPaperToView(0);
    const frame = requestAnimationFrame(() => fitPaperToView(0));
    return () => cancelAnimationFrame(frame);
  }, [designViewportFitRequestId, fitPaperToView, projectLoadId]);

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

  const toggleMirrorMode = useCallback(() => {
    if (mirrorMode) {
      setToolMode('select');
      return;
    }
    if (!project.hasSymmetry) {
      void setSymmetry({
        hasSymmetry: true,
        symAngle: 90,
        symLoc: paperCenter(project.paper.width, project.paper.height),
      });
    }
    setLayers((current) => setDesignLayerVisibility(current, 'symmetry', true));
    setToolMode('symmetry');
  }, [mirrorMode, project.hasSymmetry, project.paper.height, project.paper.width, setSymmetry, setToolMode]);

  const openSymmetryPreview = useCallback(() => {
    const scope = selectedLeafNodeIds.length > 0 ? selectedLeafNodeIds : undefined;
    setSymmetryPreview(previewSymmetryLeafPairs(scope));
  }, [previewSymmetryLeafPairs, selectedLeafNodeIds]);

  const applyOpenSymmetryPreview = useCallback(() => {
    if (!symmetryPreview) return;
    void applySymmetryLeafPairs(symmetryPreview.scopedLeafIds).then(() => setSymmetryPreview(null));
  }, [applySymmetryLeafPairs, symmetryPreview]);

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
    const loc = eventToPaper(event);
    if (mirrorMode) void addNodeWithSymmetry(loc, connectTo);
    else void addNodeAt(loc, connectTo);
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
    if (moved) {
      if (mirrorMode) void moveNodeWithSymmetry(nodeId, loc);
      else void moveNode(nodeId, loc);
    }
  };

  const onCanvasPointerMove = (event: PointerEvent<SVGSVGElement>) => {
    setHoverPoint(eventToPaper(event));
  };

  if (documentMode === 'crease-pattern') {
    return (
      <section className="panel-shell design-panel">
        <div className="panel-body document-mode-empty">
          <div className="document-mode-empty__icon" aria-hidden="true">
            <FileQuestionMark size={30} />
          </div>
          <span className="document-mode-empty__message">
            {importedCreasePattern
              ? (
                <>
                  <span className="document-mode-empty__filename">
                    {importedCreasePattern.source.filename}
                  </span>{' '}
                  is an imported crease pattern without a TreeMaker tree.
                </>
              )
              : (
                'This document does not have a TreeMaker tree.'
              )}
          </span>
          <div className="document-mode-empty__actions">
            <Button size="sm" variant="primary" onClick={() => void handleMenuAction('view.creasePattern')}>
              <ScanLine size={14} />
              View CP
            </Button>
            <Button size="sm" variant="secondary" onClick={() => void handleMenuAction('file.new')}>
              <FileText size={14} />
              New Tree
            </Button>
            <Button size="sm" variant="secondary" onClick={() => void handleMenuAction('file.open')}>
              <FolderOpen size={14} />
              Open
            </Button>
          </div>
        </div>
      </section>
    );
  }

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
                <>
                  {mirrorMode && (
                    <line
                      className="symmetry-snap-lane"
                      x1={symmetryLine.x1}
                      y1={symmetryLine.y1}
                      x2={symmetryLine.x2}
                      y2={symmetryLine.y2}
                    />
                  )}
                  <line
                    className="symmetry-line"
                    x1={symmetryLine.x1}
                    y1={symmetryLine.y1}
                    x2={symmetryLine.x2}
                    y2={symmetryLine.y2}
                  />
                </>
              )}
              {showEmptyState && (
                <foreignObject
                  className="design-empty-state"
                  x={DESIGN_PAPER_RECT.x}
                  y={DESIGN_PAPER_RECT.y}
                  width={DESIGN_PAPER_RECT.width}
                  height={DESIGN_PAPER_RECT.height}
                >
                  <div className="design-empty-state__inner" role="note">
                    <div className="design-empty-state__copy">
                      <strong>Sketch the tree behind your design</strong>
                      <span>Use branches for the flaps, limbs, and features the folded base needs.</span>
                    </div>
                  </div>
                </foreignObject>
              )}
              {symmetryHoverPreview && (
                <g className="symmetry-ghost">
                  {(() => {
                    const p1 = paperToSvg(symmetryHoverPreview.primary.from, DESIGN_PAPER_RECT);
                    const p2 = paperToSvg(symmetryHoverPreview.primary.to, DESIGN_PAPER_RECT);
                    return (
                      <>
                        <line
                          className={[
                            'symmetry-ghost-edge',
                            symmetryHoverPreview.unresolved ? 'symmetry-ghost-edge--unresolved' : '',
                          ].join(' ')}
                          x1={p1.x}
                          y1={p1.y}
                          x2={p2.x}
                          y2={p2.y}
                        />
                        <circle
                          className="symmetry-ghost-node"
                          data-snapped={symmetryHoverPreview.snapped || undefined}
                          cx={p2.x}
                          cy={p2.y}
                          r="7"
                        />
                      </>
                    );
                  })()}
                  {symmetryHoverPreview.mirror &&
                    (() => {
                      const p1 = paperToSvg(symmetryHoverPreview.mirror.from, DESIGN_PAPER_RECT);
                      const p2 = paperToSvg(symmetryHoverPreview.mirror.to, DESIGN_PAPER_RECT);
                      return (
                        <>
                          <line
                            className="symmetry-ghost-edge"
                            x1={p1.x}
                            y1={p1.y}
                            x2={p2.x}
                            y2={p2.y}
                          />
                          <circle className="symmetry-ghost-node" cx={p2.x} cy={p2.y} r="7" />
                        </>
                      );
                    })()}
                </g>
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
          mirrorMode={mirrorMode}
          hasSymmetry={project.hasSymmetry}
          onLayerChange={setLayer}
          onToggleMirror={toggleMirrorMode}
          onPairLeaves={openSymmetryPreview}
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
        {symmetryPreview && (
          <div className="symmetry-preview-popover" role="dialog" aria-label="Symmetry leaf preview">
            <div className="symmetry-preview-popover__header">
              <strong>Symmetry Preview</strong>
              <button type="button" onClick={() => setSymmetryPreview(null)}>
                Close
              </button>
            </div>
            <div className="symmetry-preview-popover__grid">
              <span>Pairs</span>
              <strong>{symmetryPreview.pairs.length}</strong>
              <span>On-axis</span>
              <strong>{symmetryPreview.onAxis.length}</strong>
              <span>Ambiguous</span>
              <strong>{symmetryPreview.ambiguous.length}</strong>
              <span>Unmatched</span>
              <strong>{symmetryPreview.unmatched.length}</strong>
            </div>
            {(symmetryPreview.ambiguous.length > 0 || symmetryPreview.unmatched.length > 0) && (
              <div className="symmetry-preview-popover__detail">
                {[...symmetryPreview.ambiguous, ...symmetryPreview.unmatched]
                  .map((item) => item.node)
                  .join(', ')}
              </div>
            )}
            <button
              className="symmetry-preview-popover__apply"
              type="button"
              disabled={symmetryPreview.pairs.length + symmetryPreview.onAxis.length === 0}
              onClick={applyOpenSymmetryPreview}
            >
              Apply
            </button>
          </div>
        )}
        <div className="design-legend">
          <span><CircleDot size={13} /> Terminal</span>
          <span><Waypoints size={13} /> Active path</span>
          <span><Plus size={13} /> Scale {formatNumber(project.scale, 3)}</span>
        </div>
      </div>
    </section>
  );
}
