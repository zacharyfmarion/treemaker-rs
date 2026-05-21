import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type PointerEvent,
} from 'react';
import { TransformComponent, TransformWrapper, type ReactZoomPanPinchRef } from 'react-zoom-pan-pinch';
import { GitBranch, Grid2X2, Magnet, ScanLine } from 'lucide-react';
import type { OristudioCpDocumentSnapshot } from '../../engine/oristudioCpTypes';
import { formatNumber, paperToSvg, type Point } from '../../lib/geometry';
import { getViewportFitScale, type ViewportSize } from '../../lib/designViewport';
import {
  CP_PAPER_RECT,
  CP_PAPER_SHADOW_RECT,
  CP_VIEWBOX_SIZE,
  CP_WORLD_RECT,
  cpLineAssignmentLabel,
  cpLineColorClass,
  cpSelectionSize,
  cpSvgPointToModel,
  getCpGridLines,
  getEditableCpModelBounds,
  modelPointToCpSvg,
  nearestCpSnapTarget,
  textCoordinate,
  type CpSnapTarget,
  type OristudioCpSelection,
} from '../../lib/creasePatternViewport';
import type { Selection, TreeProject } from '../../lib/sampleProject';
import {
  isCreaseSelected,
  isFacetSelected,
  selectionSize,
  toggleCreaseSelection,
  toggleFacetSelection,
} from '../../lib/selection';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { IconButton } from '../ui/IconButton';
import { SegmentedControl } from '../ui/SegmentedControl';
import { NextDocumentAction } from './NextDocumentAction';
import {
  isViewportInteractiveTarget,
  ViewportToolbar,
  ViewportToolbarSeparator,
} from './ViewportToolbar';

function creaseClass(fold: string, kind: string, mode: 'mvf' | 'agrh'): string {
  if (mode === 'agrh') return `crease crease--kind-${kind}`;
  return `crease crease--fold-${fold}`;
}

function formatZoom(scale: number): string {
  return `${Math.round(scale * 100)}%`;
}

export function CreasePatternPanel() {
  const svgRef = useRef<SVGSVGElement | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const transformRef = useRef<ReactZoomPanPinchRef | null>(null);
  const [zoomPercent, setZoomPercent] = useState(100);
  const [spacePressed, setSpacePressed] = useState(false);
  const [cursorModelPoint, setCursorModelPoint] = useState<Point | null>(null);
  const [snapTarget, setSnapTarget] = useState<CpSnapTarget | null>(null);

  const project = useWorkspaceStore((state) => state.project);
  const status = useWorkspaceStore((state) => state.status);
  const error = useWorkspaceStore((state) => state.error);
  const documentMode = useWorkspaceStore((state) => state.documentMode);
  const importedCreasePattern = useWorkspaceStore((state) => state.importedCreasePattern);
  const oristudioCpDocument = useWorkspaceStore((state) => state.oristudioCpDocument);
  const oristudioCpError = useWorkspaceStore((state) => state.oristudioCpError);
  const oristudioCpSelection = useWorkspaceStore((state) => state.oristudioCpSelection);
  const oristudioCpViewport = useWorkspaceStore((state) => state.oristudioCpViewport);
  const projectLoadId = useWorkspaceStore((state) => state.projectLoadId);
  const mode = useWorkspaceStore((state) => state.creaseColorMode);
  const selection = useWorkspaceStore((state) => state.selection);
  const setMode = useWorkspaceStore((state) => state.setCreaseColorMode);
  const select = useWorkspaceStore((state) => state.select);
  const setOristudioCpViewportOption = useWorkspaceStore(
    (state) => state.setOristudioCpViewportOption
  );
  const toggleOristudioCpLineSelection = useWorkspaceStore(
    (state) => state.toggleOristudioCpLineSelection
  );
  const toggleOristudioCpPointSelection = useWorkspaceStore(
    (state) => state.toggleOristudioCpPointSelection
  );
  const toggleOristudioCpCircleSelection = useWorkspaceStore(
    (state) => state.toggleOristudioCpCircleSelection
  );
  const toggleOristudioCpTextSelection = useWorkspaceStore(
    (state) => state.toggleOristudioCpTextSelection
  );
  const clearOristudioCpSelection = useWorkspaceStore((state) => state.clearOristudioCpSelection);

  const editableCp = documentMode === 'crease-pattern' ? oristudioCpDocument?.document : null;
  const editableCpSummary = oristudioCpDocument?.summary ?? null;
  const editableCpBounds = useMemo(() => getEditableCpModelBounds(editableCp), [editableCp]);
  const editableCpGridLines = useMemo(
    () =>
      editableCp
        ? getCpGridLines(
            editableCpBounds,
            editableCp.crease_pattern.grid.grid_size,
            editableCp.crease_pattern.grid.interval_grid_size
          )
        : [],
    [editableCp, editableCpBounds]
  );
  const hasEditableCreasePattern =
    !!editableCp &&
    (editableCp.crease_pattern.line_segments.length > 0 ||
      editableCp.crease_pattern.circles.length > 0 ||
      editableCp.crease_pattern.points.length > 0 ||
      editableCp.crease_pattern.texts.length > 0);
  const hasCreasePattern =
    hasEditableCreasePattern || project.creases.length > 0 || project.facets.length > 0;
  const editableSelectionSize = cpSelectionSize(oristudioCpSelection);

  const clearSelectionOnBackgroundPointerDown = (event: PointerEvent<SVGElement>) => {
    if (event.button !== 0 || spacePressed) return;
    if (editableCp && editableSelectionSize > 0) {
      clearOristudioCpSelection();
      return;
    }
    if (selectionSize(selection) === 0) return;
    select({ kind: 'tree' });
  };
  const emptyStatusLabel =
    status === 'building_crease_pattern'
      ? 'Building crease pattern'
      : status === 'optimizing'
        ? 'Optimizing scale'
        : status === 'error' && error
          ? shortStatus(error.message)
          : documentMode === 'crease-pattern'
            ? 'No imported crease pattern'
            : 'No crease pattern';
  const sourceLabel =
    documentMode === 'crease-pattern' && importedCreasePattern
      ? [
          importedCreasePattern.source.filename,
          importedCreasePattern.lineOnly ? 'View only' : 'Simulatable',
          oristudioCpDocument
            ? `Editable kernel: ${oristudioCpDocument.summary.line_segments} lines`
            : oristudioCpError
              ? `Kernel unavailable: ${shortStatus(oristudioCpError)}`
              : 'Editable kernel pending',
        ].join(' | ')
      : null;

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
    return getViewportFitScale(viewport, CP_WORLD_RECT);
  }, [getViewportSize]);

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

  const eventToEditableModelPoint = useCallback(
    (event: PointerEvent<SVGElement>): Point | null => {
      const svg = svgRef.current;
      if (!svg || !editableCp) return null;
      const bounds = svg.getBoundingClientRect();
      if (bounds.width <= 0 || bounds.height <= 0) return null;
      const svgPoint = {
        x: ((event.clientX - bounds.left) / bounds.width) * CP_VIEWBOX_SIZE,
        y: ((event.clientY - bounds.top) / bounds.height) * CP_VIEWBOX_SIZE,
      };
      return cpSvgPointToModel(svgPoint, editableCpBounds);
    },
    [editableCp, editableCpBounds]
  );

  const updateEditablePointerStatus = useCallback(
    (event: PointerEvent<SVGElement>) => {
      if (!editableCp) return;
      const modelPoint = eventToEditableModelPoint(event);
      setCursorModelPoint(modelPoint);
      setSnapTarget(
        modelPoint
          ? nearestCpSnapTarget(editableCp, modelPoint, editableCpBounds, oristudioCpViewport)
          : null
      );
    },
    [editableCp, editableCpBounds, eventToEditableModelPoint, oristudioCpViewport]
  );

  const clearEditablePointerStatus = useCallback(() => {
    setCursorModelPoint(null);
    setSnapTarget(null);
  }, []);

  const creasePatternFitKey = useMemo(
    () =>
      `${projectLoadId}:${project.creases.length}:${project.facets.length}:${editableCpSummary?.line_segments ?? 0}`,
    [editableCpSummary?.line_segments, project.creases.length, project.facets.length, projectLoadId]
  );
  const lastFittedCreasePatternRef = useRef<string | null>(null);

  const fitLoadedCreasePattern = useCallback(
    (animationTime = 0) => {
      if (!hasCreasePattern) {
        lastFittedCreasePatternRef.current = null;
        return true;
      }
      if (lastFittedCreasePatternRef.current === creasePatternFitKey) return true;
      const container = containerRef.current;
      if (!container || !transformRef.current || container.clientWidth <= 0 || container.clientHeight <= 0) {
        return false;
      }
      transformRef.current.centerView(computeFitScale(), animationTime);
      lastFittedCreasePatternRef.current = creasePatternFitKey;
      return true;
    },
    [computeFitScale, creasePatternFitKey, hasCreasePattern]
  );

  const fitLoadedCreasePatternRef = useRef(fitLoadedCreasePattern);
  useEffect(() => {
    fitLoadedCreasePatternRef.current = fitLoadedCreasePattern;
  }, [fitLoadedCreasePattern]);

  useEffect(() => {
    const container = containerRef.current;
    if (!hasCreasePattern) {
      lastFittedCreasePatternRef.current = null;
      return undefined;
    }

    let frame = requestAnimationFrame(() => fitLoadedCreasePatternRef.current(0));
    const observer =
      typeof ResizeObserver === 'undefined' || !container
        ? null
        : new ResizeObserver(() => {
            if (lastFittedCreasePatternRef.current !== creasePatternFitKey) {
              cancelAnimationFrame(frame);
              frame = requestAnimationFrame(() => fitLoadedCreasePatternRef.current(0));
            }
          });

    if (observer && container) {
      observer.observe(container);
    }
    return () => {
      cancelAnimationFrame(frame);
      observer?.disconnect();
    };
  }, [creasePatternFitKey, hasCreasePattern]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container || !hasCreasePattern) return undefined;

    const onKeyDown = (event: KeyboardEvent) => {
      const interactive = isViewportInteractiveTarget(event.target);
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
  }, [fitToView, hasCreasePattern, setActualSize]);

  return (
    <section className="panel-shell cp-panel">
      <div className="panel-toolbar">
        <div className="panel-toolbar__group">
          <span className="panel-title">Crease Pattern</span>
        </div>
        {hasCreasePattern ? (
          <div className="cp-panel__mode">
            <span className="cp-panel__mode-label">Color by</span>
            <SegmentedControl
              aria-label="Choose how crease lines are colored"
              value={mode}
              onChange={setMode}
              options={[
                {
                  value: 'agrh',
                  label: 'Crease roles',
                  icon: <ScanLine size={13} />,
                  title: 'Color by axial, gusset, ridge, hinge, and pseudohinge roles',
                },
                {
                  value: 'mvf',
                  label: 'M/V assignment',
                  icon: <GitBranch size={13} />,
                  title: 'Color by mountain, valley, flat, and border folds',
                },
              ]}
            />
          </div>
        ) : (
          <span className="panel-toolbar__meta">{emptyStatusLabel}</span>
        )}
      </div>
      {sourceLabel && <div className="panel-subtitle">{sourceLabel}</div>}
      <div
        ref={containerRef}
        className="panel-body cp-panel__body"
        data-space-pan={spacePressed || undefined}
        tabIndex={-1}
        onPointerDown={(event) => {
          if (!isViewportInteractiveTarget(event.target)) containerRef.current?.focus();
        }}
      >
        {hasCreasePattern ? (
          <>
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
                requestAnimationFrame(() => fitLoadedCreasePatternRef.current(0));
              }}
              onTransformed={(_ref, state) => setZoomPercent(Math.round(state.scale * 100))}
            >
              <TransformComponent
                wrapperStyle={{ width: '100%', height: '100%' }}
                contentStyle={{ width: 'fit-content', height: 'fit-content' }}
              >
                <svg
                  ref={svgRef}
                  className="cp-canvas"
                  viewBox={`0 0 ${CP_VIEWBOX_SIZE} ${CP_VIEWBOX_SIZE}`}
                  width={CP_VIEWBOX_SIZE}
                  height={CP_VIEWBOX_SIZE}
                  style={{ width: CP_VIEWBOX_SIZE, height: CP_VIEWBOX_SIZE }}
                  role="img"
                  aria-label="Crease pattern"
                  onPointerMove={updateEditablePointerStatus}
                  onPointerLeave={clearEditablePointerStatus}
                  onPointerDown={(event) => {
                    if (event.target === event.currentTarget) clearSelectionOnBackgroundPointerDown(event);
                  }}
                >
                  <rect
                    className="paper-shadow"
                    x={CP_PAPER_SHADOW_RECT.x}
                    y={CP_PAPER_SHADOW_RECT.y}
                    width={CP_PAPER_SHADOW_RECT.width}
                    height={CP_PAPER_SHADOW_RECT.height}
                    rx="6"
                  />
                  <rect
                    className="paper"
                    x={CP_PAPER_RECT.x}
                    y={CP_PAPER_RECT.y}
                    width={CP_PAPER_RECT.width}
                    height={CP_PAPER_RECT.height}
                    onPointerDown={clearSelectionOnBackgroundPointerDown}
                  />
                  {editableCp ? (
                    <EditableCreasePattern
                      bounds={editableCpBounds}
                      clearSelectionOnBackgroundPointerDown={clearSelectionOnBackgroundPointerDown}
                      document={editableCp}
                      gridLines={editableCpGridLines}
                      gridVisible={oristudioCpViewport.gridVisible}
                      mode={mode}
                      selection={oristudioCpSelection}
                      snapTarget={snapTarget}
                      spacePressed={spacePressed}
                      toggleCircle={toggleOristudioCpCircleSelection}
                      toggleLine={toggleOristudioCpLineSelection}
                      togglePoint={toggleOristudioCpPointSelection}
                      toggleText={toggleOristudioCpTextSelection}
                    />
                  ) : (
                    <GeneratedCreasePattern
                      clearSelectionOnBackgroundPointerDown={clearSelectionOnBackgroundPointerDown}
                      mode={mode}
                      project={project}
                      select={select}
                      selection={selection}
                      spacePressed={spacePressed}
                    />
                  )}
                </svg>
              </TransformComponent>
            </TransformWrapper>
            <ViewportToolbar
              ariaLabel="Crease pattern viewport controls"
              zoomPercent={zoomPercent}
              zoomIn={() => transformRef.current?.zoomIn(0.35, 120)}
              zoomOut={() => transformRef.current?.zoomOut(0.35, 120)}
              fitToView={() => fitToView()}
              setActualSize={setActualSize}
              setZoomLevel={setZoomLevel}
            >
              {editableCp && (
                <>
                  <ViewportToolbarSeparator />
                  <IconButton
                    size="sm"
                    variant="toolbar"
                    title="Grid"
                    isActive={oristudioCpViewport.gridVisible}
                    onClick={() =>
                      setOristudioCpViewportOption('gridVisible', !oristudioCpViewport.gridVisible)
                    }
                  >
                    <Grid2X2 size={14} />
                  </IconButton>
                  <IconButton
                    size="sm"
                    variant="toolbar"
                    title="Snap"
                    isActive={
                      oristudioCpViewport.snapToGrid ||
                      oristudioCpViewport.snapToVertices ||
                      oristudioCpViewport.snapToLines
                    }
                    onClick={() => {
                      const enabled =
                        oristudioCpViewport.snapToGrid ||
                        oristudioCpViewport.snapToVertices ||
                        oristudioCpViewport.snapToLines;
                      setOristudioCpViewportOption('snapToGrid', !enabled);
                      setOristudioCpViewportOption('snapToVertices', !enabled);
                      setOristudioCpViewportOption('snapToLines', !enabled);
                    }}
                  >
                    <Magnet size={14} />
                  </IconButton>
                </>
              )}
            </ViewportToolbar>
            <div className="viewport-status-readout">
              <span>{formatZoom(zoomPercent / 100)}</span>
              {editableCp && editableCpSummary && (
                <span>{editableCpSummary.line_segments} lines</span>
              )}
              {editableCp && cursorModelPoint && (
                <span>
                  {formatNumber(cursorModelPoint.x, 2)}, {formatNumber(cursorModelPoint.y, 2)}
                </span>
              )}
              {editableCp && snapTarget && <span>Snap {snapTarget.label}</span>}
              {editableCp && editableSelectionSize > 0 && (
                <span>{editableSelectionSize} selected</span>
              )}
            </div>
          </>
        ) : (
          <div className="cp-panel__empty">
            <span title={status === 'error' ? error?.message : undefined}>{emptyStatusLabel}</span>
            <NextDocumentAction />
          </div>
        )}
      </div>
    </section>
  );
}

interface EditableCreasePatternProps {
  bounds: ReturnType<typeof getEditableCpModelBounds>;
  clearSelectionOnBackgroundPointerDown: (event: PointerEvent<SVGElement>) => void;
  document: OristudioCpDocumentSnapshot;
  gridLines: ReturnType<typeof getCpGridLines>;
  gridVisible: boolean;
  mode: 'mvf' | 'agrh';
  selection: OristudioCpSelection;
  snapTarget: CpSnapTarget | null;
  spacePressed: boolean;
  toggleCircle: (id: number, additive?: boolean) => void;
  toggleLine: (id: number, additive?: boolean) => void;
  togglePoint: (id: number, additive?: boolean) => void;
  toggleText: (id: number, additive?: boolean) => void;
}

function EditableCreasePattern({
  bounds,
  clearSelectionOnBackgroundPointerDown,
  document,
  gridLines,
  gridVisible,
  mode,
  selection,
  snapTarget,
  spacePressed,
  toggleCircle,
  toggleLine,
  togglePoint,
  toggleText,
}: EditableCreasePatternProps) {
  return (
    <>
      {gridVisible &&
        gridLines.map((line) => {
          const a = modelPointToCpSvg(line.a, bounds);
          const b = modelPointToCpSvg(line.b, bounds);
          return (
            <line
              key={line.id}
              className={['cp-grid-line', line.major ? 'cp-grid-line--major' : ''].join(' ')}
              x1={a.x}
              y1={a.y}
              x2={b.x}
              y2={b.y}
            />
          );
        })}
      {document.crease_pattern.line_segments.map((line, index) => {
        const id = index + 1;
        const a = modelPointToCpSvg(line.a, bounds);
        const b = modelPointToCpSvg(line.b, bounds);
        return (
          <line
            key={id}
            className={[
              cpLineColorClass(line.color, mode),
              selection.lines.includes(id) ? 'crease--selected' : '',
            ].join(' ')}
            data-cp-line-id={id}
            x1={a.x}
            y1={a.y}
            x2={b.x}
            y2={b.y}
            aria-label={`Editable ${cpLineAssignmentLabel(line.color)} line ${id}`}
            onClick={(event) => {
              if (spacePressed) return;
              event.stopPropagation();
              toggleLine(id, event.shiftKey || event.metaKey || event.ctrlKey);
            }}
          />
        );
      })}
      {document.crease_pattern.points.map((point, index) => {
        const id = index + 1;
        const svgPoint = modelPointToCpSvg(point, bounds);
        return (
          <circle
            key={id}
            className={[
              'cp-point',
              selection.points.includes(id) ? 'cp-point--selected' : '',
            ].join(' ')}
            cx={svgPoint.x}
            cy={svgPoint.y}
            r="4"
            onClick={(event) => {
              if (spacePressed) return;
              event.stopPropagation();
              togglePoint(id, event.shiftKey || event.metaKey || event.ctrlKey);
            }}
          />
        );
      })}
      {document.crease_pattern.circles.map((circle, index) => {
        const id = index + 1;
        const center = modelPointToCpSvg({ x: circle.x, y: circle.y }, bounds);
        const radius =
          (circle.r / Math.max(bounds.spanX, bounds.spanY)) *
          Math.min(CP_PAPER_RECT.width, CP_PAPER_RECT.height);
        return (
          <circle
            key={id}
            className={[
              'cp-circle',
              selection.circles.includes(id) ? 'cp-circle--selected' : '',
            ].join(' ')}
            cx={center.x}
            cy={center.y}
            r={Math.max(1, radius)}
            onClick={(event) => {
              if (spacePressed) return;
              event.stopPropagation();
              toggleCircle(id, event.shiftKey || event.metaKey || event.ctrlKey);
            }}
          />
        );
      })}
      {document.crease_pattern.texts.map((text, index) => {
        const id = index + 1;
        const position = modelPointToCpSvg(
          { x: textCoordinate(text.x), y: textCoordinate(text.y) },
          bounds
        );
        return (
          <text
            key={id}
            className={['cp-text', selection.texts.includes(id) ? 'cp-text--selected' : ''].join(
              ' '
            )}
            x={position.x}
            y={position.y}
            onClick={(event) => {
              if (spacePressed) return;
              event.stopPropagation();
              toggleText(id, event.shiftKey || event.metaKey || event.ctrlKey);
            }}
          >
            {text.text}
          </text>
        );
      })}
      {snapTarget && (
        <circle
          className="cp-snap-target"
          cx={modelPointToCpSvg(snapTarget.point, bounds).x}
          cy={modelPointToCpSvg(snapTarget.point, bounds).y}
          r="5"
        />
      )}
      <rect
        className="paper-border"
        x={CP_PAPER_RECT.x}
        y={CP_PAPER_RECT.y}
        width={CP_PAPER_RECT.width}
        height={CP_PAPER_RECT.height}
        onPointerDown={clearSelectionOnBackgroundPointerDown}
      />
    </>
  );
}

interface GeneratedCreasePatternProps {
  clearSelectionOnBackgroundPointerDown: (event: PointerEvent<SVGElement>) => void;
  mode: 'mvf' | 'agrh';
  project: TreeProject;
  select: (selection: Selection) => void;
  selection: Selection;
  spacePressed: boolean;
}

function GeneratedCreasePattern({
  clearSelectionOnBackgroundPointerDown,
  mode,
  project,
  select,
  selection,
  spacePressed,
}: GeneratedCreasePatternProps) {
  return (
    <>
      {project.facets.map((facet) => {
        const points = facet.vertices
          .map((point) => paperToSvg(point, CP_PAPER_RECT))
          .map((point) => `${point.x},${point.y}`)
          .join(' ');
        return (
          <polygon
            key={facet.id}
            className={[
              `facet facet--${facet.color}`,
              isFacetSelected(selection, facet.id) ? 'facet--selected' : '',
            ].join(' ')}
            points={points}
            onClick={(event) => {
              if (spacePressed) return;
              select(
                event.shiftKey || event.metaKey || event.ctrlKey
                  ? toggleFacetSelection(selection, facet.id)
                  : { kind: 'facet', id: facet.id }
              );
            }}
          />
        );
      })}
      {project.creases.map((crease) => {
        const a = paperToSvg(crease.vertices[0], CP_PAPER_RECT);
        const b = paperToSvg(crease.vertices[1], CP_PAPER_RECT);
        return (
          <line
            key={crease.id}
            className={[
              creaseClass(crease.fold, crease.kind, mode),
              isCreaseSelected(selection, crease.id) ? 'crease--selected' : '',
            ].join(' ')}
            x1={a.x}
            y1={a.y}
            x2={b.x}
            y2={b.y}
            onClick={(event) => {
              if (spacePressed) return;
              select(
                event.shiftKey || event.metaKey || event.ctrlKey
                  ? toggleCreaseSelection(selection, crease.id)
                  : { kind: 'crease', id: crease.id }
              );
            }}
          />
        );
      })}
      <rect
        className="paper-border"
        x={CP_PAPER_RECT.x}
        y={CP_PAPER_RECT.y}
        width={CP_PAPER_RECT.width}
        height={CP_PAPER_RECT.height}
        onPointerDown={clearSelectionOnBackgroundPointerDown}
      />
    </>
  );
}

function shortStatus(message: string): string {
  const trimmed = message.trim();
  if (!trimmed) return 'Crease pattern unavailable';
  const sentence = trimmed.split(/[.;]\s+/u)[0] ?? trimmed;
  return sentence.length > 54 ? `${sentence.slice(0, 51)}...` : sentence;
}
