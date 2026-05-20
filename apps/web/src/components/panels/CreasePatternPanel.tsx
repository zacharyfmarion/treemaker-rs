import type { PointerEvent } from 'react';
import { GitBranch, ScanLine } from 'lucide-react';
import { paperToSvg, type PlotRect } from '../../lib/geometry';
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

const VIEWBOX = 720;
const PAPER_RECT: PlotRect = { x: 66, y: 54, width: 588, height: 588 };

function creaseClass(fold: string, kind: string, mode: 'mvf' | 'agrh'): string {
  if (mode === 'agrh') return `crease crease--kind-${kind}`;
  return `crease crease--fold-${fold}`;
}

export function CreasePatternPanel() {
  const project = useWorkspaceStore((state) => state.project);
  const status = useWorkspaceStore((state) => state.status);
  const error = useWorkspaceStore((state) => state.error);
  const documentMode = useWorkspaceStore((state) => state.documentMode);
  const importedCreasePattern = useWorkspaceStore((state) => state.importedCreasePattern);
  const mode = useWorkspaceStore((state) => state.creaseColorMode);
  const selection = useWorkspaceStore((state) => state.selection);
  const setMode = useWorkspaceStore((state) => state.setCreaseColorMode);
  const select = useWorkspaceStore((state) => state.select);
  const hasCreasePattern = project.creases.length > 0 || project.facets.length > 0;
  const clearSelectionOnBackgroundPointerDown = (event: PointerEvent<SVGElement>) => {
    if (event.button !== 0 || selectionSize(selection) === 0) return;
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
      ? `${importedCreasePattern.source.filename} | ${importedCreasePattern.lineOnly ? 'View only' : 'Simulatable'}`
      : null;

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
      <div className="panel-body cp-panel__body">
        {hasCreasePattern ? (
          <svg
            className="cp-canvas"
            viewBox={`0 0 ${VIEWBOX} ${VIEWBOX}`}
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
