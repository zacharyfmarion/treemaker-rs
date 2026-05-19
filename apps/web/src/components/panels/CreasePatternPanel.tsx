import { GitBranch, ScanLine } from 'lucide-react';
import { paperToSvg, type PlotRect } from '../../lib/geometry';
import {
  isCreaseSelected,
  isFacetSelected,
  toggleCreaseSelection,
  toggleFacetSelection,
} from '../../lib/selection';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { SegmentedControl } from '../ui/SegmentedControl';

const VIEWBOX = 720;
const PAPER_RECT: PlotRect = { x: 66, y: 54, width: 588, height: 588 };

function creaseClass(fold: string, kind: string, mode: 'mvf' | 'agrh'): string {
  if (mode === 'agrh') return `crease crease--kind-${kind}`;
  return `crease crease--fold-${fold}`;
}

export function CreasePatternPanel() {
  const project = useWorkspaceStore((state) => state.project);
  const mode = useWorkspaceStore((state) => state.creaseColorMode);
  const selection = useWorkspaceStore((state) => state.selection);
  const setMode = useWorkspaceStore((state) => state.setCreaseColorMode);
  const select = useWorkspaceStore((state) => state.select);

  return (
    <section className="panel-shell cp-panel">
      <div className="panel-toolbar">
        <div className="panel-toolbar__group">
          <span className="panel-title">Crease Pattern</span>
        </div>
        <div className="cp-panel__mode">
          <span className="cp-panel__mode-label">Color by</span>
          <SegmentedControl
            aria-label="Choose how crease lines are colored"
            value={mode}
            onChange={setMode}
            options={[
              {
                value: 'mvf',
                label: 'Fold types',
                icon: <ScanLine size={13} />,
                title: 'Color by mountain, valley, flat, and border folds',
              },
              {
                value: 'agrh',
                label: 'Crease roles',
                icon: <GitBranch size={13} />,
                title: 'Color by axial, gusset, ridge, hinge, and pseudohinge roles',
              },
            ]}
          />
        </div>
      </div>
      <div className="panel-body cp-panel__body">
        <svg className="cp-canvas" viewBox={`0 0 ${VIEWBOX} ${VIEWBOX}`} role="img" aria-label="Crease pattern">
          <rect className="paper-shadow" x="56" y="44" width="608" height="608" rx="6" />
          <rect
            className="paper"
            x={PAPER_RECT.x}
            y={PAPER_RECT.y}
            width={PAPER_RECT.width}
            height={PAPER_RECT.height}
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
          />
        </svg>
      </div>
    </section>
  );
}
