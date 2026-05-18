import { describe, expect, it } from 'vitest';
import { createOrigamiSimulator, detectWebGlSupport, prepareFoldModel } from '../src/index.js';
import { createThreeOrigamiRenderer } from '../src/three.js';
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

  it('leaves a flat model still when the target fold percent is zero', () => {
    const prepared = prepareFoldModel(makeBookFoldFixture());
    const simulator = createOrigamiSimulator({ model: prepared, options: { foldPercent: 0 } });
    const before = simulator.readFrame().positions;
    const after = simulator.step(64).positions;

    expect(maxPositionDelta(before, after)).toBeLessThan(1e-6);
    simulator.dispose();
  });

  it('reports WebGL availability without throwing in node', () => {
    expect(detectWebGlSupport()).toBe(false);
  });
});

describe('createThreeOrigamiRenderer', () => {
  it('updates geometry attributes in place from simulator frames', () => {
    const prepared = prepareFoldModel(makeBookFoldFixture());
    const simulator = createOrigamiSimulator({ model: prepared, options: { foldPercent: 100 } });
    const renderer = createThreeOrigamiRenderer(prepared);
    const position = renderer.mesh.geometry.getAttribute('position');
    const frame = simulator.step(32);

    renderer.update(frame);

    expect(renderer.mesh.geometry.getAttribute('position')).toBe(position);
    expect(maxPositionDelta((position.array as Float32Array), frame.positions)).toBe(0);

    renderer.dispose();
    simulator.dispose();
  });
});
