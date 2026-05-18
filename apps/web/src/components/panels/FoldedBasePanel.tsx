import { useEffect, useMemo, useState } from 'react';
import { Layers3, RefreshCw } from 'lucide-react';
import type { FoldedBaseSnapshot, FoldedBaseVertex } from '../../engine/types';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { Button } from '../ui/Button';
import { IconButton } from '../ui/IconButton';

const VIEWBOX = 720;
const PADDING = 62;

export function FoldedBasePanel() {
  const creaseCount = useWorkspaceStore((state) => state.project.creases.length);
  const foldArtifacts = useWorkspaceStore((state) => state.foldArtifacts);
  const foldArtifactError = useWorkspaceStore((state) => state.foldArtifactError);
  const refreshFoldArtifacts = useWorkspaceStore((state) => state.refreshFoldArtifacts);
  const buildCreasePattern = useWorkspaceStore((state) => state.buildCreasePattern);
  const [loading, setLoading] = useState(false);

  const foldedBase = foldArtifacts?.folded_base ?? null;
  const foldedBaseError = foldArtifacts?.folded_base_error ?? foldArtifactError;

  useEffect(() => {
    if (creaseCount === 0 || foldArtifacts) return;
    let cancelled = false;
    setLoading(true);
    void refreshFoldArtifacts().finally(() => {
      if (!cancelled) setLoading(false);
    });
    return () => {
      cancelled = true;
    };
  }, [creaseCount, foldArtifacts, refreshFoldArtifacts]);

  const statusLabel =
    creaseCount === 0
      ? 'No crease pattern'
      : loading
        ? 'Loading'
        : foldedBase
          ? `${foldedBase.vertices.length} vertices | ${foldedBase.facets.length} facets`
          : shortStatus(foldedBaseError ?? 'Folded base unavailable');

  return (
    <section className="panel-shell folded-base-panel">
      <div className="panel-toolbar">
        <div className="panel-toolbar__group">
          <Layers3 size={14} />
          <span className="panel-title">Folded Base</span>
        </div>
        <div className="panel-toolbar__group">
          <span className="panel-toolbar__meta">{statusLabel}</span>
          <IconButton
            size="sm"
            title="Refresh"
            tooltipSide="bottom"
            onClick={() => {
              setLoading(true);
              void refreshFoldArtifacts().finally(() => setLoading(false));
            }}
            disabled={creaseCount === 0}
          >
            <RefreshCw size={14} />
          </IconButton>
        </div>
      </div>
      <div className="panel-body folded-base-panel__body">
        {foldedBase ? (
          <FoldedBaseSvg snapshot={foldedBase} />
        ) : (
          <div className="folded-base-panel__empty">
            <span title={foldedBaseError ?? undefined}>{statusLabel}</span>
            {creaseCount === 0 && (
              <Button size="sm" variant="primary" onClick={() => void buildCreasePattern()}>
                Build
              </Button>
            )}
          </div>
        )}
      </div>
    </section>
  );
}

function FoldedBaseSvg({ snapshot }: { snapshot: FoldedBaseSnapshot }) {
  const projection = useMemo(() => createProjection(snapshot.vertices), [snapshot.vertices]);
  const verticesById = useMemo(
    () => new Map(snapshot.vertices.map((vertex) => [vertex.id, vertex])),
    [snapshot.vertices]
  );
  const facets = useMemo(
    () => [...snapshot.facets].sort((a, b) => a.order - b.order || a.id - b.id),
    [snapshot.facets]
  );

  return (
    <svg
      className="folded-base-canvas"
      viewBox={`0 0 ${VIEWBOX} ${VIEWBOX}`}
      role="img"
      aria-label="Folded base"
    >
      <rect className="paper-shadow" x="48" y="46" width="624" height="624" rx="6" />
      <rect className="folded-base-plane" x="54" y="54" width="612" height="612" />
      {facets.map((facet) => {
        const points = facet.vertices
          .map((id) => verticesById.get(id))
          .filter(isVertex)
          .map((vertex) => projection(vertex))
          .map((point) => `${point.x},${point.y}`)
          .join(' ');
        if (!points) return null;
        return (
          <polygon
            key={facet.id}
            className={`folded-base-facet folded-base-facet--color-${facet.color}`}
            points={points}
          />
        );
      })}
      {snapshot.creases.map((crease) => {
        const a = verticesById.get(crease.vertices[0]);
        const b = verticesById.get(crease.vertices[1]);
        if (!a || !b) return null;
        const p1 = projection(a);
        const p2 = projection(b);
        return (
          <line
            key={crease.id}
            className={`folded-base-crease folded-base-crease--fold-${crease.fold}`}
            x1={p1.x}
            y1={p1.y}
            x2={p2.x}
            y2={p2.y}
          />
        );
      })}
      {snapshot.vertices.map((vertex) => {
        const point = projection(vertex);
        return (
          <circle
            key={vertex.id}
            className={vertex.is_border ? 'folded-base-vertex folded-base-vertex--border' : 'folded-base-vertex'}
            cx={point.x}
            cy={point.y}
            r={vertex.is_border ? 3.2 : 2.4}
          />
        );
      })}
    </svg>
  );
}

function createProjection(vertices: FoldedBaseVertex[]) {
  const bounds = vertices.reduce(
    (acc, vertex) => ({
      minX: Math.min(acc.minX, vertex.loc.x),
      maxX: Math.max(acc.maxX, vertex.loc.x),
      minY: Math.min(acc.minY, vertex.loc.y),
      maxY: Math.max(acc.maxY, vertex.loc.y),
    }),
    { minX: Infinity, maxX: -Infinity, minY: Infinity, maxY: -Infinity }
  );
  const minX = Number.isFinite(bounds.minX) ? bounds.minX : 0;
  const maxX = Number.isFinite(bounds.maxX) ? bounds.maxX : 1;
  const minY = Number.isFinite(bounds.minY) ? bounds.minY : 0;
  const maxY = Number.isFinite(bounds.maxY) ? bounds.maxY : 1;
  const spanX = Math.max(0.001, maxX - minX);
  const spanY = Math.max(0.001, maxY - minY);
  const scale = Math.min((VIEWBOX - PADDING * 2) / spanX, (VIEWBOX - PADDING * 2) / spanY);
  const offsetX = (VIEWBOX - spanX * scale) / 2;
  const offsetY = (VIEWBOX - spanY * scale) / 2;

  return (vertex: FoldedBaseVertex) => ({
    x: offsetX + (vertex.loc.x - minX) * scale,
    y: VIEWBOX - offsetY - (vertex.loc.y - minY) * scale,
  });
}

function isVertex(vertex: FoldedBaseVertex | undefined): vertex is FoldedBaseVertex {
  return vertex !== undefined;
}

function shortStatus(message: string): string {
  const trimmed = message.trim();
  if (!trimmed) return 'Folded base unavailable';
  const sentence = trimmed.split(/[.;]\s+/u)[0] ?? trimmed;
  return sentence.length > 54 ? `${sentence.slice(0, 51)}...` : sentence;
}
