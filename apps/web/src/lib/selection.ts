import type { Selection, TreeProject } from './sampleProject';

export function emptyMultiSelection(): Extract<Selection, { kind: 'multi' }> {
  return {
    kind: 'multi',
    nodes: [],
    edges: [],
    paths: [],
    creases: [],
    facets: [],
    conditions: [],
  };
}

function uniqueSorted(ids: number[]): number[] {
  return Array.from(new Set(ids)).sort((a, b) => a - b);
}

export function selectedNodeIds(selection: Selection): number[] {
  if (selection.kind === 'node') return [selection.id];
  if (selection.kind === 'multi') return selection.nodes;
  return [];
}

export function selectedEdgeIds(selection: Selection): number[] {
  if (selection.kind === 'edge') return [selection.id];
  if (selection.kind === 'multi') return selection.edges;
  return [];
}

export function selectedPathIds(selection: Selection): number[] {
  if (selection.kind === 'path') return [selection.id];
  if (selection.kind === 'multi') return selection.paths;
  return [];
}

export function selectedCreaseIds(selection: Selection): number[] {
  if (selection.kind === 'crease') return [selection.id];
  if (selection.kind === 'multi') return selection.creases;
  return [];
}

export function selectedFacetIds(selection: Selection): number[] {
  if (selection.kind === 'facet') return [selection.id];
  if (selection.kind === 'multi') return selection.facets;
  return [];
}

export function selectedConditionIds(selection: Selection): number[] {
  if (selection.kind === 'condition') return [selection.id];
  if (selection.kind === 'multi') return selection.conditions;
  return [];
}

export function isNodeSelected(selection: Selection, id: number): boolean {
  return selectedNodeIds(selection).includes(id);
}

export function isEdgeSelected(selection: Selection, id: number): boolean {
  return selectedEdgeIds(selection).includes(id);
}

export function isPathSelected(selection: Selection, id: number): boolean {
  return selectedPathIds(selection).includes(id);
}

export function isCreaseSelected(selection: Selection, id: number): boolean {
  return selectedCreaseIds(selection).includes(id);
}

export function isFacetSelected(selection: Selection, id: number): boolean {
  return selectedFacetIds(selection).includes(id);
}

export function isConditionSelected(selection: Selection, id: number): boolean {
  return selectedConditionIds(selection).includes(id);
}

export function selectedEditablePartCount(selection: Selection): number {
  return selectedNodeIds(selection).length + selectedEdgeIds(selection).length;
}

export function toggleNodeSelection(selection: Selection, id: number): Selection {
  const multi = selection.kind === 'multi' ? selection : emptyMultiSelection();
  const nodes = multi.nodes.includes(id)
    ? multi.nodes.filter((nodeId) => nodeId !== id)
    : uniqueSorted([...multi.nodes, id]);
  const next = { ...multi, nodes };
  return selectionSize(next) === 0 ? { kind: 'tree' } : next;
}

export function toggleEdgeSelection(selection: Selection, id: number): Selection {
  const multi = selection.kind === 'multi' ? selection : emptyMultiSelection();
  const edges = multi.edges.includes(id)
    ? multi.edges.filter((edgeId) => edgeId !== id)
    : uniqueSorted([...multi.edges, id]);
  const next = { ...multi, edges };
  return selectionSize(next) === 0 ? { kind: 'tree' } : next;
}

export function toggleCreaseSelection(selection: Selection, id: number): Selection {
  const multi = selection.kind === 'multi' ? selection : emptyMultiSelection();
  const creases = multi.creases.includes(id)
    ? multi.creases.filter((creaseId) => creaseId !== id)
    : uniqueSorted([...multi.creases, id]);
  const next = { ...multi, creases };
  return selectionSize(next) === 0 ? { kind: 'tree' } : next;
}

export function toggleFacetSelection(selection: Selection, id: number): Selection {
  const multi = selection.kind === 'multi' ? selection : emptyMultiSelection();
  const facets = multi.facets.includes(id)
    ? multi.facets.filter((facetId) => facetId !== id)
    : uniqueSorted([...multi.facets, id]);
  const next = { ...multi, facets };
  return selectionSize(next) === 0 ? { kind: 'tree' } : next;
}

export function selectionSize(selection: Selection): number {
  switch (selection.kind) {
    case 'tree':
      return 0;
    case 'multi':
      return (
        selection.nodes.length +
        selection.edges.length +
        selection.paths.length +
        selection.creases.length +
        selection.facets.length +
        selection.conditions.length
      );
    default:
      return 1;
  }
}

export function selectionSummary(selection: Selection): string {
  if (selection.kind !== 'multi') return selection.kind;
  const parts = [
    selection.nodes.length ? `${selection.nodes.length} nodes` : '',
    selection.edges.length ? `${selection.edges.length} edges` : '',
    selection.paths.length ? `${selection.paths.length} paths` : '',
    selection.creases.length ? `${selection.creases.length} creases` : '',
    selection.facets.length ? `${selection.facets.length} facets` : '',
    selection.conditions.length ? `${selection.conditions.length} conditions` : '',
  ].filter(Boolean);
  return parts.join(', ') || 'selection';
}

export function selectEverything(project: TreeProject): Selection {
  const selection = {
    kind: 'multi' as const,
    nodes: project.nodes.map((node) => node.id),
    edges: project.edges.map((edge) => edge.id),
    paths: project.paths.map((path) => path.id),
    creases: project.creases.map((crease) => crease.id),
    facets: project.facets.map((facet) => facet.id),
    conditions: project.conditions.map((condition) => condition.id),
  };
  return selectionSize(selection) === 0 ? { kind: 'tree' } : selection;
}
