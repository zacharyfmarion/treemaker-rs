import { describe, expect, it } from 'vitest';
import type { TreeProject } from './sampleProject';
import {
  detectSymmetryLeafPairs,
  distanceToSymmetryAxis,
  projectOntoSymmetryAxis,
  reflectPointAcrossSymmetryAxis,
  symmetrySide,
} from './symmetryAuthoring';

function project(
  nodes: TreeProject['nodes'],
  conditions: TreeProject['conditions'] = []
): TreeProject {
  return {
    title: 'Symmetry test',
    paper: { width: 1, height: 1, symLoc: { x: 0.5, y: 0.5 }, symAngle: 90 },
    scale: 0.1,
    hasSymmetry: true,
    nodes,
    edges: [],
    paths: [],
    creases: [],
    facets: [],
    conditions,
  };
}

function node(id: number, x: number, y: number, isLeaf = true): TreeProject['nodes'][number] {
  return {
    id,
    label: `n${id}`,
    loc: { x, y },
    isLeaf,
    isPinned: false,
    isConditioned: false,
  };
}

describe('symmetry authoring helpers', () => {
  it('projects, reflects, and classifies points around the axis', () => {
    const axis = { loc: { x: 0.5, y: 0.5 }, angle: 90 };

    expect(projectOntoSymmetryAxis({ x: 0.25, y: 0.7 }, axis)).toEqual({
      x: 0.5,
      y: 0.7,
    });
    expect(reflectPointAcrossSymmetryAxis({ x: 0.25, y: 0.7 }, axis)).toEqual({
      x: 0.75,
      y: 0.7,
    });
    expect(distanceToSymmetryAxis({ x: 0.47, y: 0.25 }, axis)).toBeCloseTo(0.03);
    expect(symmetrySide({ x: 0.25, y: 0.7 }, axis)).toBe(-1);
    expect(symmetrySide({ x: 0.5, y: 0.7 }, axis)).toBe(0);
    expect(symmetrySide({ x: 0.75, y: 0.7 }, axis)).toBe(1);
  });

  it('detects mutual nearest leaf pairs and on-axis leaves', () => {
    const preview = detectSymmetryLeafPairs(
      project([
        node(1, 0.5, 0.5, false),
        node(2, 0.2, 0.25),
        node(3, 0.8, 0.25),
        node(4, 0.504, 0.82),
      ])
    );

    expect(preview.pairs).toHaveLength(1);
    expect(preview.pairs[0]).toMatchObject({ node1: 2, node2: 3 });
    expect(preview.pairs[0].distance).toBeCloseTo(0);
    expect(preview.onAxis).toHaveLength(1);
    expect(preview.onAxis[0].node).toBe(4);
    expect(preview.onAxis[0].distance).toBeCloseTo(0.004);
    expect(preview.ambiguous).toEqual([]);
    expect(preview.unmatched).toEqual([]);
  });

  it('reports ambiguous and unmatched leaves without pairing internal nodes', () => {
    const preview = detectSymmetryLeafPairs(
      project([
        node(1, 0.5, 0.5, false),
        node(2, 0.2, 0.3),
        node(3, 0.802, 0.3),
        node(4, 0.805, 0.3),
        node(5, 0.9, 0.9, false),
        node(6, 0.1, 0.85),
      ])
    );

    expect(preview.pairs).toEqual([]);
    expect(preview.ambiguous).toEqual([{ node: 2, candidates: [3, 4] }]);
    expect(preview.unmatched).toEqual([{ node: 6 }]);
    expect(preview.scopedLeafIds).toEqual([2, 3, 4, 6]);
  });

  it('skips existing pair and on-axis conditions', () => {
    const preview = detectSymmetryLeafPairs(
      project(
        [
          node(1, 0.5, 0.5, false),
          node(2, 0.2, 0.25),
          node(3, 0.8, 0.25),
          node(4, 0.5, 0.82),
        ],
        [
          {
            id: 1,
            isFeasible: true,
            kind: { type: 'nodes_paired', node1: 2, node2: 3 },
          },
          {
            id: 2,
            isFeasible: true,
            kind: { type: 'node_symmetric', node: 4 },
          },
        ]
      )
    );

    expect(preview.pairs).toEqual([]);
    expect(preview.onAxis).toEqual([]);
    expect(preview.unmatched).toEqual([]);
  });
});
