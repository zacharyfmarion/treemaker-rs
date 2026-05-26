import { useEffect, useMemo, useState } from 'react';
import { Eye, GitBranch, Layers3 } from 'lucide-react';
import type { FoldedBaseSnapshot, FoldedBaseVertex } from '../../engine/types';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { IconButton } from '../ui/IconButton';
import { NextDocumentAction } from './NextDocumentAction';

const VIEWBOX = 720;
const PADDING = 62;

interface FoldedBaseViewOptions {
  wireframe: boolean;
  translucent: boolean;
}

export function FoldedBasePanel() {
  const creaseCount = useWorkspaceStore((state) => state.project.creases.length);
  const status = useWorkspaceStore((state) => state.status);
  const documentMode = useWorkspaceStore((state) => state.documentMode);
  const editableCpDocument = useWorkspaceStore((state) => state.oristudioCpDocument?.document ?? null);
  const foldArtifacts = useWorkspaceStore((state) => state.foldArtifacts);
  const foldArtifactError = useWorkspaceStore((state) => state.foldArtifactError);
  const foldArtifactStatus = useWorkspaceStore((state) => state.foldArtifactStatus);
  const ensureFoldArtifacts = useWorkspaceStore((state) => state.ensureFoldArtifacts);
  const [viewOptions, setViewOptions] = useState<FoldedBaseViewOptions>({
    wireframe: false,
    translucent: false,
  });

  const foldedBase = foldArtifacts?.folded_base ?? null;
  const foldedBaseError = foldArtifacts?.folded_base_error ?? foldArtifactError;

  useEffect(() => {
    const needsTreeArtifacts = documentMode === 'tree' && creaseCount > 0;
    const needsEditableCpArtifacts =
      documentMode === 'crease-pattern' && editableCpDocument !== null;
    if (!needsTreeArtifacts && !needsEditableCpArtifacts) return;
    if (foldArtifactStatus !== 'idle' && foldArtifactStatus !== 'stale') return;
    void ensureFoldArtifacts();
  }, [
    creaseCount,
    documentMode,
    editableCpDocument,
    ensureFoldArtifacts,
    foldArtifactStatus,
  ]);

  const emptyStatus =
    documentMode === 'tree' && creaseCount === 0
      ? status === 'building_crease_pattern'
        ? 'Building crease pattern'
        : 'No crease pattern'
      : foldArtifactStatus === 'loading'
        ? 'Updating folded base'
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
        {foldedBase && (
          <div className="panel-toolbar__group">
            <div className="folded-base-view-controls" aria-label="Folded base view options">
              <IconButton
                size="sm"
                variant="toolbar"
                title="Wireframe"
                tooltipSide="bottom"
                isActive={viewOptions.wireframe}
                onClick={() =>
                  setViewOptions((current) => ({
                    ...current,
                    wireframe: !current.wireframe,
                  }))
                }
              >
                <GitBranch size={14} />
              </IconButton>
              <IconButton
                size="sm"
                variant="toolbar"
                title="Translucent Layers"
                tooltipSide="bottom"
                isActive={viewOptions.translucent}
                onClick={() =>
                  setViewOptions((current) => ({
                    ...current,
                    translucent: !current.translucent,
                  }))
                }
              >
                <Eye size={14} />
              </IconButton>
            </div>
          </div>
        )}
      </div>
      <div className="panel-body folded-base-panel__body">
        {foldedBase ? (
          <FoldedBaseSvg snapshot={foldedBase} viewOptions={viewOptions} />
        ) : (
          <div className="folded-base-panel__empty">
            <span title={foldedBaseError ?? undefined}>{emptyStatus}</span>
            {documentMode === 'tree' && creaseCount === 0 && <NextDocumentAction />}
          </div>
        )}
      </div>
    </section>
  );
}

function FoldedBaseSvg({
  snapshot,
  viewOptions,
}: {
  snapshot: FoldedBaseSnapshot;
  viewOptions: FoldedBaseViewOptions;
}) {
  const projection = useMemo(() => createProjection(snapshot.vertices), [snapshot.vertices]);
  const verticesById = useMemo(
    () => new Map(snapshot.vertices.map((vertex) => [vertex.id, vertex])),
    [snapshot.vertices]
  );
  const facets = useMemo(
    () => [...snapshot.facets].sort((a, b) => a.order - b.order || a.id - b.id),
    [snapshot.facets]
  );
  const showCreases = viewOptions.wireframe;
  const visibleCreases = showCreases
    ? snapshot.creases
    : snapshot.creases.filter((crease) => crease.fold === 3);

  return (
    <svg
      className="folded-base-canvas"
      data-wireframe={viewOptions.wireframe || undefined}
      data-translucent={viewOptions.translucent || undefined}
      viewBox={`0 0 ${VIEWBOX} ${VIEWBOX}`}
      role="img"
      aria-label="Folded base"
    >
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
      {visibleCreases.map((crease) => {
        const a = verticesById.get(crease.vertices[0]);
        const b = verticesById.get(crease.vertices[1]);
        if (!a || !b) return null;
        const p1 = projection(a);
        const p2 = projection(b);
        return (
          <line
            key={crease.id}
            className={
              showCreases
                ? `folded-base-crease folded-base-crease--fold-${crease.fold}`
                : 'folded-base-outline'
            }
            x1={p1.x}
            y1={p1.y}
            x2={p2.x}
            y2={p2.y}
          />
        );
      })}
      {viewOptions.wireframe &&
        snapshot.vertices.map((vertex) => {
          const point = projection(vertex);
          return (
            <circle
              key={vertex.id}
              className={
                vertex.is_border
                  ? 'folded-base-vertex folded-base-vertex--border'
                  : 'folded-base-vertex'
              }
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
