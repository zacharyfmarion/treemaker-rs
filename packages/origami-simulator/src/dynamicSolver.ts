import { OrigamiModel } from './model.js';
import type { SimulationFrame, SimulatorOptions } from './types.js';

const EPSILON = 1e-6;
const TWO_PI = Math.PI * 2;

const DEFAULT_OPTIONS: Required<SimulatorOptions> = {
  foldPercent: 0,
  axialStiffness: 20,
  creaseStiffness: 0.7,
  panelStiffness: 0.7,
  faceStiffness: 0.2,
  damping: 0.45,
  timeStep: 0,
  stepsPerFrame: 100,
  autoRender: true,
};

export class DynamicSolver {
  readonly model: OrigamiModel;
  options: Required<SimulatorOptions>;
  private currentStep = 0;
  private readonly forces: Float32Array;
  private readonly theta: Float32Array;
  private readonly nodeCreases: NodeCreaseRef[][];
  private readonly nodeFaces: NodeFaceRef[][];
  private readonly nominalAngles: Vec3[];

  constructor(model: OrigamiModel, options: SimulatorOptions = {}) {
    this.model = model;
    this.options = { ...DEFAULT_OPTIONS, ...options };
    this.forces = new Float32Array(model.positions.length);
    this.theta = new Float32Array(model.prepared.creaseParams.length);
    this.nodeCreases = buildNodeCreases(model);
    this.nodeFaces = buildNodeFaces(model);
    this.nominalAngles = buildNominalAngles(model);
  }

  setFoldPercent(percent: number): void {
    this.options.foldPercent = Math.max(-100, Math.min(100, percent));
  }

  setMaterial(options: Partial<SimulatorOptions>): void {
    this.options = {
      ...this.options,
      ...options,
      foldPercent:
        options.foldPercent === undefined
          ? this.options.foldPercent
          : Math.max(-100, Math.min(100, options.foldPercent)),
    };
  }

  reset(): void {
    this.currentStep = 0;
    this.theta.fill(0);
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
    const dt = this.timeStep();
    const normals = this.model.computeFaceNormals();
    const theta = this.updateCreaseAngles(normals);
    const creaseGeometry = this.computeCreaseGeometry();

    this.accumulateBeamForces();
    this.accumulateCreaseForces(normals, theta, creaseGeometry);
    this.accumulateFaceForces(normals);

    const velocityDamping = 1 - Math.max(0, Math.min(0.99, this.options.damping)) * dt * 2;
    for (let index = 0; index < this.model.positions.length; index += 1) {
      const velocity = (this.model.velocities[index] + this.forces[index] * dt) * velocityDamping;
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
    const axialStiffness = Math.max(0, this.options.axialStiffness);
    const damping = Math.max(0, this.options.damping);
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
      const stiffness = axialStiffness / rest;
      const beamDamping = damping * 2 * Math.sqrt(stiffness);
      const stretchScale = 1 - rest / length;
      const vx = (this.model.velocities[b] ?? 0) - (this.model.velocities[a] ?? 0);
      const vy = (this.model.velocities[b + 1] ?? 0) - (this.model.velocities[a + 1] ?? 0);
      const vz = (this.model.velocities[b + 2] ?? 0) - (this.model.velocities[a + 2] ?? 0);
      const fx = dx * stretchScale * stiffness + vx * beamDamping;
      const fy = dy * stretchScale * stiffness + vy * beamDamping;
      const fz = dz * stretchScale * stiffness + vz * beamDamping;
      this.addForce(edge[0], fx, fy, fz);
      this.addForce(edge[1], -fx, -fy, -fz);
    }
  }

  private accumulateCreaseForces(normals: Float32Array, theta: Float32Array, geometry: CreaseGeometry[]): void {
    const foldScale = this.options.foldPercent / 100;
    for (let vertex = 0; vertex < this.nodeCreases.length; vertex += 1) {
      for (const ref of this.nodeCreases[vertex] ?? []) {
        const crease = this.model.prepared.creaseParams[ref.creaseIndex];
        const geo = geometry[ref.creaseIndex];
        if (!crease || !geo || !geo.enabled) continue;
        const targetTheta = ((crease.targetAngle * Math.PI) / 180) * foldScale;
        const stiffness =
          (crease.targetAngle === 0 ? this.options.panelStiffness : this.options.creaseStiffness) *
          this.model.edgeRestLength(crease.edge);
        const angularForce = stiffness * (targetTheta - (theta[ref.creaseIndex] ?? 0));

        if (ref.nodeNumber > 2) {
          const normal1 = vectorAt(normals, crease.face1);
          const normal2 = vectorAt(normals, crease.face2);
          let coef1 = geo.coef1;
          let coef2 = geo.coef2;
          if (ref.nodeNumber === 3) {
            coef1 = 1 - coef1;
            coef2 = 1 - coef2;
          }
          this.addForce(
            vertex,
            -angularForce * ((coef1 / geo.height1) * normal1[0] + (coef2 / geo.height2) * normal2[0]),
            -angularForce * ((coef1 / geo.height1) * normal1[1] + (coef2 / geo.height2) * normal2[1]),
            -angularForce * ((coef1 / geo.height1) * normal1[2] + (coef2 / geo.height2) * normal2[2])
          );
        } else {
          const normalIndex = ref.nodeNumber === 1 ? crease.face1 : crease.face2;
          const momentArm = ref.nodeNumber === 1 ? geo.height1 : geo.height2;
          const normal = vectorAt(normals, normalIndex);
          const scale = angularForce / momentArm;
          this.addForce(vertex, scale * normal[0], scale * normal[1], scale * normal[2]);
        }
      }
    }
  }

  private accumulateFaceForces(normals: Float32Array): void {
    const stiffness = Math.max(0, this.options.faceStiffness);
    if (stiffness <= 0) return;
    for (let vertex = 0; vertex < this.nodeFaces.length; vertex += 1) {
      const vertexPosition = pointAt(this.model.positions, vertex);
      for (const ref of this.nodeFaces[vertex] ?? []) {
        const nominal = this.nominalAngles[ref.faceIndex];
        if (!nominal) continue;

        const a = ref.vertexOffset === 0 ? vertexPosition : pointAt(this.model.positions, ref.face[0]);
        const b = ref.vertexOffset === 1 ? vertexPosition : pointAt(this.model.positions, ref.face[1]);
        const c = ref.vertexOffset === 2 ? vertexPosition : pointAt(this.model.positions, ref.face[2]);
        const ab = subtract(b, a);
        const ac = subtract(c, a);
        const bc = subtract(c, b);
        const lengthAB = magnitude(ab);
        const lengthAC = magnitude(ac);
        const lengthBC = magnitude(bc);
        if (lengthAB < EPSILON || lengthAC < EPSILON || lengthBC < EPSILON) continue;

        const unitAB = scale(ab, 1 / lengthAB);
        const unitAC = scale(ac, 1 / lengthAC);
        const unitBC = scale(bc, 1 / lengthBC);
        const angles: Vec3 = [
          Math.acos(clamp(dot(unitAB, unitAC), -1, 1)),
          Math.acos(clamp(-dot(unitAB, unitBC), -1, 1)),
          Math.acos(clamp(dot(unitAC, unitBC), -1, 1)),
        ];
        const diff: Vec3 = [
          (nominal[0] - angles[0]) * stiffness,
          (nominal[1] - angles[1]) * stiffness,
          (nominal[2] - angles[2]) * stiffness,
        ];
        const normal = vectorAt(normals, ref.faceIndex);

        if (ref.vertexOffset === 0) {
          const normalCrossAC = scale(cross(normal, unitAC), 1 / lengthAC);
          const normalCrossAB = scale(cross(normal, unitAB), 1 / lengthAB);
          this.addForceVec(vertex, subtract(scale(subtract(normalCrossAC, normalCrossAB), -diff[0]), scale(normalCrossAB, diff[1])));
          this.addForceVec(vertex, scale(normalCrossAC, diff[2]));
        } else if (ref.vertexOffset === 1) {
          const normalCrossAB = scale(cross(normal, unitAB), 1 / lengthAB);
          const normalCrossBC = scale(cross(normal, unitBC), 1 / lengthBC);
          this.addForceVec(vertex, scale(normalCrossAB, -diff[0]));
          this.addForceVec(vertex, scale(add(normalCrossAB, normalCrossBC), diff[1]));
          this.addForceVec(vertex, scale(normalCrossBC, -diff[2]));
        } else {
          const normalCrossAC = scale(cross(normal, unitAC), 1 / lengthAC);
          const normalCrossBC = scale(cross(normal, unitBC), 1 / lengthBC);
          this.addForceVec(vertex, scale(normalCrossAC, diff[0]));
          this.addForceVec(vertex, scale(normalCrossBC, -diff[1]));
          this.addForceVec(vertex, scale(subtract(normalCrossBC, normalCrossAC), diff[2]));
        }
      }
    }
  }

  private addForce(vertex: number, fx: number, fy: number, fz: number): void {
    const index = vertex * 3;
    this.forces[index] += fx;
    this.forces[index + 1] += fy;
    this.forces[index + 2] += fz;
  }

  private addForceVec(vertex: number, force: Vec3): void {
    this.addForce(vertex, force[0], force[1], force[2]);
  }

  private timeStep(): number {
    if (this.options.timeStep > 0) return this.options.timeStep;
    let maxFrequency = 0;
    const axialStiffness = Math.max(0, this.options.axialStiffness);
    for (let edgeIndex = 0; edgeIndex < this.model.prepared.edgesVertices.length; edgeIndex += 1) {
      const rest = this.model.edgeRestLength(edgeIndex);
      const stiffness = axialStiffness / rest;
      maxFrequency = Math.max(maxFrequency, Math.sqrt(stiffness));
    }
    if (maxFrequency <= EPSILON) return 1 / 60;
    return clamp(0.9 / (TWO_PI * maxFrequency), 1 / 5000, 1 / 30);
  }

  private updateCreaseAngles(normals: Float32Array): Float32Array {
    for (let creaseIndex = 0; creaseIndex < this.model.prepared.creaseParams.length; creaseIndex += 1) {
      const crease = this.model.prepared.creaseParams[creaseIndex];
      if (!crease) continue;
      const edge = this.model.prepared.edgesVertices[crease.edge];
      if (!edge) continue;
      const normal1 = vectorAt(normals, crease.face1);
      const normal2 = vectorAt(normals, crease.face2);
      const creaseVector = normalize(subtract(pointAt(this.model.positions, edge[1]), pointAt(this.model.positions, edge[0])));
      const theta = Math.atan2(dot(cross(normal1, creaseVector), normal2), clamp(dot(normal1, normal2), -1, 1));
      let diff = theta - (this.theta[creaseIndex] ?? 0);
      if (diff < -5) diff += TWO_PI;
      else if (diff > 5) diff -= TWO_PI;
      this.theta[creaseIndex] = (this.theta[creaseIndex] ?? 0) + diff;
    }
    return this.theta;
  }

  private computeCreaseGeometry(): CreaseGeometry[] {
    return this.model.prepared.creaseParams.map((crease) => {
      const edge = this.model.prepared.edgesVertices[crease.edge];
      if (!edge) return DISABLED_CREASE_GEOMETRY;
      const node1 = pointAt(this.model.positions, crease.vertex1);
      const node2 = pointAt(this.model.positions, crease.vertex2);
      const node3 = pointAt(this.model.positions, edge[0]);
      const node4 = pointAt(this.model.positions, edge[1]);
      const creaseVector = subtract(node4, node3);
      const creaseLength = magnitude(creaseVector);
      if (creaseLength < EPSILON) return DISABLED_CREASE_GEOMETRY;
      const unitCrease = scale(creaseVector, 1 / creaseLength);
      const vector1 = subtract(node1, node3);
      const vector2 = subtract(node2, node3);
      const projection1 = dot(unitCrease, vector1);
      const projection2 = dot(unitCrease, vector2);
      const height1 = Math.sqrt(Math.max(0, dot(vector1, vector1) - projection1 * projection1));
      const height2 = Math.sqrt(Math.max(0, dot(vector2, vector2) - projection2 * projection2));
      if (height1 < EPSILON || height2 < EPSILON) return DISABLED_CREASE_GEOMETRY;
      return {
        enabled: true,
        height1,
        height2,
        coef1: projection1 / creaseLength,
        coef2: projection2 / creaseLength,
      };
    });
  }
}

type Vec3 = [number, number, number];

interface NodeCreaseRef {
  creaseIndex: number;
  nodeNumber: 1 | 2 | 3 | 4;
}

interface NodeFaceRef {
  faceIndex: number;
  face: number[];
  vertexOffset: 0 | 1 | 2;
}

interface CreaseGeometry {
  enabled: boolean;
  height1: number;
  height2: number;
  coef1: number;
  coef2: number;
}

const DISABLED_CREASE_GEOMETRY: CreaseGeometry = {
  enabled: false,
  height1: EPSILON,
  height2: EPSILON,
  coef1: 0,
  coef2: 0,
};

function vectorAt(source: Float32Array, index: number): Vec3 {
  const offset = index * 3;
  return [source[offset] ?? 0, source[offset + 1] ?? 0, source[offset + 2] ?? 0];
}

function pointAt(source: Float32Array, vertex: number): Vec3 {
  const offset = vertex * 3;
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

function add(a: Vec3, b: Vec3): Vec3 {
  return [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
}

function subtract(a: Vec3, b: Vec3): Vec3 {
  return [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
}

function scale(vector: Vec3, scalar: number): Vec3 {
  return [vector[0] * scalar, vector[1] * scalar, vector[2] * scalar];
}

function magnitude(vector: Vec3): number {
  return Math.hypot(vector[0], vector[1], vector[2]);
}

function normalize(vector: Vec3): Vec3 {
  const length = magnitude(vector);
  if (length <= EPSILON) return [1, 0, 0];
  return [vector[0] / length, vector[1] / length, vector[2] / length];
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function buildNodeCreases(model: OrigamiModel): NodeCreaseRef[][] {
  const refs = Array.from({ length: model.prepared.vertexCount }, (): NodeCreaseRef[] => []);
  model.prepared.creaseParams.forEach((crease, creaseIndex) => {
    const edge = model.prepared.edgesVertices[crease.edge];
    if (!edge) return;
    refs[crease.vertex1]?.push({ creaseIndex, nodeNumber: 1 });
    refs[crease.vertex2]?.push({ creaseIndex, nodeNumber: 2 });
    refs[edge[0]]?.push({ creaseIndex, nodeNumber: 3 });
    refs[edge[1]]?.push({ creaseIndex, nodeNumber: 4 });
  });
  return refs;
}

function buildNodeFaces(model: OrigamiModel): NodeFaceRef[][] {
  const refs = Array.from({ length: model.prepared.vertexCount }, (): NodeFaceRef[] => []);
  model.prepared.facesVertices.forEach((face, faceIndex) => {
    if (face.length !== 3) return;
    face.forEach((vertex, vertexOffset) => {
      refs[vertex]?.push({
        faceIndex,
        face,
        vertexOffset: vertexOffset as 0 | 1 | 2,
      });
    });
  });
  return refs;
}

function buildNominalAngles(model: OrigamiModel): Vec3[] {
  return model.prepared.facesVertices.map((face) => {
    const a = pointAt(model.originalPositions, face[0] ?? 0);
    const b = pointAt(model.originalPositions, face[1] ?? 0);
    const c = pointAt(model.originalPositions, face[2] ?? 0);
    const ab = normalize(subtract(b, a));
    const ac = normalize(subtract(c, a));
    const bc = normalize(subtract(c, b));
    return [
      Math.acos(clamp(dot(ab, ac), -1, 1)),
      Math.acos(clamp(-dot(ab, bc), -1, 1)),
      Math.acos(clamp(dot(ac, bc), -1, 1)),
    ];
  });
}
