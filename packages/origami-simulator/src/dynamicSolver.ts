import { OrigamiModel } from './model.js';
import type { SimulationFrame, SimulatorOptions } from './types.js';

const EPSILON = 1e-6;

const DEFAULT_OPTIONS: Required<SimulatorOptions> = {
  foldPercent: 0,
  axialStiffness: 20,
  creaseStiffness: 0.7,
  panelStiffness: 0.7,
  damping: 0.45,
  timeStep: 1 / 60,
  stepsPerFrame: 8,
  autoRender: true,
};

export class DynamicSolver {
  readonly model: OrigamiModel;
  options: Required<SimulatorOptions>;
  private currentStep = 0;
  private readonly forces: Float32Array;

  constructor(model: OrigamiModel, options: SimulatorOptions = {}) {
    this.model = model;
    this.options = { ...DEFAULT_OPTIONS, ...options };
    this.forces = new Float32Array(model.positions.length);
  }

  setFoldPercent(percent: number): void {
    this.options.foldPercent = Math.max(-100, Math.min(100, percent));
  }

  setMaterial(options: Partial<SimulatorOptions>): void {
    this.options = { ...this.options, ...options };
  }

  reset(): void {
    this.currentStep = 0;
    this.model.reset();
  }

  step(numSteps = this.options.stepsPerFrame): SimulationFrame {
    for (let i = 0; i < numSteps; i += 1) {
      this.solveStep();
    }
    this.model.applyStrainColors(0.05);
    return this.readFrame();
  }

  readFrame(): SimulationFrame {
    return {
      positions: this.model.positions.slice(),
      colors: this.model.colors.slice(),
      indices: this.model.prepared.indices,
      diagnostics: this.model.diagnostics(),
      step: this.currentStep,
      foldPercent: this.options.foldPercent,
    };
  }

  private solveStep(): void {
    this.forces.fill(0);
    this.accumulateBeamForces();
    this.accumulateCreaseForces();
    this.accumulateTargetGuide();

    const damping = Math.max(0, Math.min(0.99, this.options.damping));
    const dt = this.options.timeStep;

    for (let index = 0; index < this.model.positions.length; index += 1) {
      const velocity = (this.model.velocities[index] + this.forces[index] * dt) * (1 - damping * dt);
      this.model.velocities[index] = velocity;
      this.model.positions[index] += velocity * dt;
      if (!Number.isFinite(this.model.positions[index])) {
        this.model.positions[index] = this.model.originalPositions[index] ?? 0;
        this.model.velocities[index] = 0;
      }
    }
    this.currentStep += 1;
  }

  private accumulateBeamForces(): void {
    const stiffness = this.options.axialStiffness;
    for (let edgeIndex = 0; edgeIndex < this.model.prepared.edgesVertices.length; edgeIndex += 1) {
      const edge = this.model.prepared.edgesVertices[edgeIndex];
      if (!edge) continue;
      const a = edge[0] * 3;
      const b = edge[1] * 3;
      const dx = (this.model.positions[b] ?? 0) - (this.model.positions[a] ?? 0);
      const dy = (this.model.positions[b + 1] ?? 0) - (this.model.positions[a + 1] ?? 0);
      const dz = (this.model.positions[b + 2] ?? 0) - (this.model.positions[a + 2] ?? 0);
      const length = Math.max(EPSILON, Math.hypot(dx, dy, dz));
      const rest = this.model.edgeRestLength(edgeIndex);
      const scale = (stiffness * (length - rest)) / rest;
      const fx = (dx / length) * scale;
      const fy = (dy / length) * scale;
      const fz = (dz / length) * scale;
      this.addForce(edge[0], fx, fy, fz);
      this.addForce(edge[1], -fx, -fy, -fz);
    }
  }

  private accumulateCreaseForces(): void {
    const normals = this.model.computeFaceNormals();
    const foldScale = this.options.foldPercent / 100;
    for (let creaseIndex = 0; creaseIndex < this.model.prepared.creaseParams.length; creaseIndex += 1) {
      const crease = this.model.prepared.creaseParams[creaseIndex];
      if (!crease) continue;
      const edge = this.model.prepared.edgesVertices[crease.edge];
      if (!edge) continue;

      const normal1 = vectorAt(normals, crease.face1);
      const normal2 = vectorAt(normals, crease.face2);
      const creaseVector = this.edgeDirection(edge[0], edge[1]);
      const dotNormals = clamp(dot(normal1, normal2), -1, 1);
      const theta = Math.atan2(dot(cross(normal1, creaseVector), normal2), dotNormals);
      const targetTheta = ((crease.targetAngle * Math.PI) / 180) * foldScale;
      const diff = theta - targetTheta;
      const stiffness = this.options.creaseStiffness * this.model.edgeRestLength(crease.edge);
      const scale = stiffness * diff;

      const arm1 = this.model.creaseMomentArm(creaseIndex, 0);
      const arm2 = this.model.creaseMomentArm(creaseIndex, 1);
      const partial1: Vec3 = [normal1[0] / arm1, normal1[1] / arm1, normal1[2] / arm1];
      const partial2: Vec3 = [normal2[0] / arm2, normal2[1] / arm2, normal2[2] / arm2];

      this.addForce(crease.vertex1, -partial1[0] * scale, -partial1[1] * scale, -partial1[2] * scale);
      this.addForce(crease.vertex2, -partial2[0] * scale, -partial2[1] * scale, -partial2[2] * scale);
      const rx = ((partial1[0] + partial2[0]) * scale) / 2;
      const ry = ((partial1[1] + partial2[1]) * scale) / 2;
      const rz = ((partial1[2] + partial2[2]) * scale) / 2;
      this.addForce(edge[0], rx, ry, rz);
      this.addForce(edge[1], rx, ry, rz);
    }
  }

  private accumulateTargetGuide(): void {
    if (this.options.panelStiffness <= 0) return;
    const target = this.model.computeTarget(this.options.foldPercent);
    const stiffness = this.options.panelStiffness * 0.12;
    for (let index = 0; index < this.model.positions.length; index += 1) {
      this.forces[index] += (target[index] - this.model.positions[index]) * stiffness;
    }
  }

  private addForce(vertex: number, fx: number, fy: number, fz: number): void {
    const index = vertex * 3;
    this.forces[index] += fx;
    this.forces[index + 1] += fy;
    this.forces[index + 2] += fz;
  }

  private edgeDirection(aVertex: number, bVertex: number): Vec3 {
    const a = aVertex * 3;
    const b = bVertex * 3;
    return normalize([
      (this.model.positions[b] ?? 0) - (this.model.positions[a] ?? 0),
      (this.model.positions[b + 1] ?? 0) - (this.model.positions[a + 1] ?? 0),
      (this.model.positions[b + 2] ?? 0) - (this.model.positions[a + 2] ?? 0),
    ]);
  }
}

type Vec3 = [number, number, number];

function vectorAt(source: Float32Array, index: number): Vec3 {
  const offset = index * 3;
  return [source[offset] ?? 0, source[offset + 1] ?? 0, source[offset + 2] ?? 0];
}

function dot(a: Vec3, b: Vec3): number {
  return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

function cross(a: Vec3, b: Vec3): Vec3 {
  return [
    a[1] * b[2] - a[2] * b[1],
    a[2] * b[0] - a[0] * b[2],
    a[0] * b[1] - a[1] * b[0],
  ];
}

function normalize(vector: Vec3): Vec3 {
  const length = Math.hypot(vector[0], vector[1], vector[2]);
  if (length <= EPSILON) return [1, 0, 0];
  return [vector[0] / length, vector[1] / length, vector[2] / length];
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
