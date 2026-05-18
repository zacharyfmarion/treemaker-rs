import { describe, expect, it } from 'vitest';
import { createOrigamiSimulator, detectWebGlSupport, prepareFoldModel } from '../src/index.js';
import { makeBookFoldFixture, maxPositionDelta } from '../src/testing.js';

describe('prepareFoldModel', () => {
  it('normalizes FOLD data and extracts crease parameters', () => {
    const prepared = prepareFoldModel(makeBookFoldFixture());

    expect(prepared.vertexCount).toBe(4);
    expect(prepared.faceCount).toBe(2);
    expect(prepared.positions[1]).toBe(0);
    expect(prepared.positions[2]).toBe(0);
    expect(prepared.positions[5]).toBe(0);
    expect(prepared.edgesAssignment[4]).toBe('M');
    expect(prepared.edgesFoldAngle[4]).toBe(-180);
    expect(prepared.creaseParams).toHaveLength(1);
    expect(prepared.creaseParams[0]).toMatchObject({ edge: 4, targetAngle: -180 });
  });

  it('triangulates quads and adds flat facet edges', () => {
    const prepared = prepareFoldModel({
      vertices_coords: [
        [0, 0],
        [1, 0],
        [1, 1],
        [0, 1],
      ],
      edges_vertices: [
        [0, 1],
        [1, 2],
        [2, 3],
        [3, 0],
      ],
      edges_assignment: ['B', 'B', 'B', 'B'],
      edges_foldAngle: [null, null, null, null],
      faces_vertices: [[0, 1, 2, 3]],
    });

    expect(prepared.facesVertices).toHaveLength(2);
    expect(prepared.edgesVertices).toHaveLength(5);
    expect(prepared.edgesAssignment[4]).toBe('F');
    expect(prepared.edgesFoldAngle[4]).toBe(0);
  });
});

describe('createOrigamiSimulator', () => {
  it('steps deterministically without requiring WebGL', () => {
    const prepared = prepareFoldModel(makeBookFoldFixture());
    const simulator = createOrigamiSimulator({ model: prepared, options: { foldPercent: 100 } });
    const before = simulator.readFrame().positions;
    const after = simulator.step(32).positions;

    expect(maxPositionDelta(before, after)).toBeGreaterThan(0);
    expect(simulator.readFrame().diagnostics.usedCpuFallback).toBe(true);

    simulator.dispose();
    expect(() => simulator.step()).toThrow(/disposed/);
  });

  it('reports WebGL availability without throwing in node', () => {
    expect(detectWebGlSupport()).toBe(false);
  });
});
