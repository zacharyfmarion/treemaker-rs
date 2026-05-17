import { describe, expect, it } from 'vitest';
import { createSampleProject } from './sampleProject';
import {
  isEdgeSelected,
  isNodeSelected,
  selectEverything,
  selectionSummary,
  toggleEdgeSelection,
  toggleNodeSelection,
} from './selection';

describe('selection helpers', () => {
  it('toggles node and edge multi-selection', () => {
    const withNode = toggleNodeSelection({ kind: 'tree' }, 2);
    expect(isNodeSelected(withNode, 2)).toBe(true);

    const withEdge = toggleEdgeSelection(withNode, 1);
    expect(isNodeSelected(withEdge, 2)).toBe(true);
    expect(isEdgeSelected(withEdge, 1)).toBe(true);
    expect(selectionSummary(withEdge)).toBe('1 nodes, 1 edges');

    expect(toggleNodeSelection(withEdge, 2)).toEqual({
      kind: 'multi',
      nodes: [],
      edges: [1],
      paths: [],
      creases: [],
      facets: [],
    });
  });

  it('selects every project part', () => {
    const selection = selectEverything(createSampleProject());
    expect(selection.kind).toBe('multi');
    if (selection.kind !== 'multi') return;
    expect(selection.nodes).toHaveLength(4);
    expect(selection.edges).toHaveLength(3);
    expect(selection.paths).toHaveLength(3);
    expect(selection.creases).toHaveLength(6);
    expect(selection.facets).toHaveLength(3);
  });
});
