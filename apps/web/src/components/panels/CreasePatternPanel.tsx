import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type PointerEvent,
} from 'react';
import { TransformComponent, TransformWrapper, type ReactZoomPanPinchRef } from 'react-zoom-pan-pinch';
import { GitBranch, ScanLine } from 'lucide-react';
import { paperToSvg, type PlotRect } from '../../lib/geometry';
import { getViewportFitScale, type ViewportSize } from '../../lib/designViewport';
import {
  isCreaseSelected,
  isFacetSelected,
  selectionSize,
  toggleCreaseSelection,
  toggleFacetSelection,
} from '../../lib/selection';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { SegmentedControl } from '../ui/SegmentedControl';
import { NextDocumentAction } from './NextDocumentAction';
import { isViewportInteractiveTarget, ViewportToolbar } from './ViewportToolbar';

const VIEWBOX = 720;
const WORLD_RECT: PlotRect = { x: 0, y: 0, width: VIEWBOX, height: VIEWBOX };
const PAPER_RECT: PlotRect = { x: 66, y: 54, width: 588, height: 588 };

function creaseClass(fold: string, kind: string, mode: 'mvf' | 'agrh'): string {
  if (mode === 'agrh') return `crease crease--kind-${kind}`;
  return `crease crease--fold-${fold}`;
}

function formatZoom(scale: number): string {
  return `${Math.round(scale * 100)}%`;
}

export function CreasePatternPanel() {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const transformRef = useRef<ReactZoomPanPinchRef | null>(null);
  const [zoomPercent, setZoomPercent] = useState(100);
  const [spacePressed, setSpacePressed] = useState(false);
  const project = useWorkspaceStore((state) => state.project);
  const status = useWorkspaceStore((state) => state.status);
  const error = useWorkspaceStore((state) => state.error);
  const documentMode = useWorkspaceStore((state) => state.documentMode);
  const importedCreasePattern = useWorkspaceStore((state) => state.importedCreasePattern);
  const oristudioCpDocument = useWorkspaceStore((state) => state.oristudioCpDocument);
  const oristudioCpError = useWorkspaceStore((state) => state.oristudioCpError);
  const projectLoadId = useWorkspaceStore((state) => state.projectLoadId);
  const mode = useWorkspaceStore((state) => state.creaseColorMode);
  const selection = useWorkspaceStore((state) => state.selection);
  const setMode = useWorkspaceStore((state) => state.setCreaseColorMode);
  const select = useWorkspaceStore((state) => state.select);
  const hasCreasePattern = project.creases.length > 0 || project.facets.length > 0;
  const clearSelectionOnBackgroundPointerDown = (event: PointerEvent<SVGElement>) => {
    if (event.button !== 0 || spacePressed || selectionSize(selection) === 0) return;
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
    return getViewportFitScale(viewport, WORLD_RECT);
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

  const creasePatternFitKey = useMemo(
    () => `${projectLoadId}:${project.creases.length}:${project.facets.length}`,
    [project.creases.length, project.facets.length, projectLoadId]
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
                  className="cp-canvas"
                  viewBox={`0 0 ${VIEWBOX} ${VIEWBOX}`}
                  width={VIEWBOX}
                  height={VIEWBOX}
                  style={{ width: VIEWBOX, height: VIEWBOX }}
                  role="img"
                  aria-label="Crease pattern"
                  onPointerDown={(event) => {
                    if (event.target === event.currentTarget) clearSelectionOnBackgroundPointerDown(event);
                  }}
                >
                  <rect className="paper-shadow" x="56" y="44" width="608" height="608" rx="6" />
                  <rect
                    className="paper"
                    x={PAPER_RECT.x}
                    y={PAPER_RECT.y}
                    width={PAPER_RECT.width}
                    height={PAPER_RECT.height}
                    onPointerDown={clearSelectionOnBackgroundPointerDown}
                  />
                  {project.facets.map((facet) => {
                    const points = facet.vertices
                      .map((point) => paperToSvg(point, PAPER_RECT))
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
                    const a = paperToSvg(crease.vertices[0], PAPER_RECT);
                    const b = paperToSvg(crease.vertices[1], PAPER_RECT);
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
                    x={PAPER_RECT.x}
                    y={PAPER_RECT.y}
                    width={PAPER_RECT.width}
                    height={PAPER_RECT.height}
                    onPointerDown={clearSelectionOnBackgroundPointerDown}
                  />
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
            />
            <div className="viewport-status-readout">
              <span>{formatZoom(zoomPercent / 100)}</span>
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

function shortStatus(message: string): string {
  const trimmed = message.trim();
  if (!trimmed) return 'Crease pattern unavailable';
  const sentence = trimmed.split(/[.;]\s+/u)[0] ?? trimmed;
  return sentence.length > 54 ? `${sentence.slice(0, 51)}...` : sentence;
}
