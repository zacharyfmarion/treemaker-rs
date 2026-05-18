import { distanceToLine2D, edgeLength, normalizePoint } from './geometry.js';
import type { PreparedOrigamiModel, SimulatorDiagnostics } from './types.js';

export class OrigamiModel {
  readonly prepared: PreparedOrigamiModel;
  readonly originalPositions: Float32Array;
  readonly positions: Float32Array;
  readonly velocities: Float32Array;
  readonly colors: Float32Array;
  private readonly originalEdgeLengths: Float32Array;

  constructor(prepared: PreparedOrigamiModel) {
    this.prepared = prepared;
    this.originalPositions = prepared.originalPositions.slice();
    this.positions = prepared.originalPositions.slice();
    this.velocities = new Float32Array(this.positions.length);
    this.colors = prepared.colors.slice();
    this.originalEdgeLengths = new Float32Array(
      prepared.edgesVertices.map((edge) => edgeLength(this.originalPositions, edge))
    );
  }

  reset(): void {
    this.positions.set(this.originalPositions);
    this.velocities.fill(0);
  }

  computeTarget(foldPercent: number): Float32Array {
    const target = this.originalPositions.slice();
    const normalizedPercent = foldPercent / 100;
    const originalPoints = this.pointTuples(this.originalPositions);

    for (const crease of this.prepared.creaseParams) {
      const edge = this.prepared.edgesVertices[crease.edge];
      if (!edge) continue;
      const a = originalPoints[edge[0]];
      const b = originalPoints[edge[1]];
      const v1 = originalPoints[crease.vertex1];
      const v2 = originalPoints[crease.vertex2];
      if (!a || !b || !v1 || !v2) continue;

      const theta = (crease.targetAngle * normalizedPercent * Math.PI) / 180;
      const h1 = Math.sin(theta / 2) * distanceToLine2D(v1, a, b);
      const h2 = Math.sin(theta / 2) * distanceToLine2D(v2, a, b);
      target[crease.vertex1 * 3 + 1] -= h1;
      target[crease.vertex2 * 3 + 1] += h2;
    }

    return target;
  }

  applyStrainColors(maxStrain: number): void {
    this.colors.fill(0.75);
    if (maxStrain <= 0) return;
    for (let edgeIndex = 0; edgeIndex < this.prepared.edgesVertices.length; edgeIndex += 1) {
      const edge = this.prepared.edgesVertices[edgeIndex];
      if (!edge) continue;
      const base = this.originalEdgeLengths[edgeIndex] || 1;
      const strain = Math.abs(edgeLength(this.positions, edge) / base - 1);
      const heat = Math.min(1, strain / maxStrain);
      for (const vertex of edge) {
        const offset = vertex * 3;
        this.colors[offset] = 0.3 + 0.7 * heat;
        this.colors[offset + 1] = 0.65 * (1 - heat);
        this.colors[offset + 2] = 0.85 * (1 - heat);
      }
    }
  }

  diagnostics(): SimulatorDiagnostics {
    let total = 0;
    let max = 0;
    for (let edgeIndex = 0; edgeIndex < this.prepared.edgesVertices.length; edgeIndex += 1) {
      const edge = this.prepared.edgesVertices[edgeIndex];
      if (!edge) continue;
      const base = this.originalEdgeLengths[edgeIndex] || 1;
      const strain = Math.abs(edgeLength(this.positions, edge) / base - 1);
      total += strain;
      max = Math.max(max, strain);
    }
    return {
      ...this.prepared.diagnostics,
      maxEdgeStrain: max,
      averageEdgeStrain: this.prepared.edgesVertices.length ? total / this.prepared.edgesVertices.length : 0,
    };
  }

  private pointTuples(source: Float32Array): Array<[number, number, number]> {
    const points: Array<[number, number, number]> = [];
    for (let index = 0; index < source.length; index += 3) {
      points.push(normalizePoint([source[index] ?? 0, source[index + 1] ?? 0, source[index + 2] ?? 0]));
    }
    return points;
  }
}
