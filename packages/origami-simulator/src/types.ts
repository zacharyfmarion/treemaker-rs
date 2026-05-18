export type FoldAssignment = 'B' | 'M' | 'V' | 'F' | 'U' | 'C' | 'J';

export interface FoldDocument {
  file_spec?: number;
  file_creator?: string;
  file_author?: string;
  frame_title?: string;
  frame_classes?: string[];
  vertices_coords: number[][];
  edges_vertices: [number, number][];
  edges_assignment?: FoldAssignment[];
  edges_foldAngle?: Array<number | null>;
  edges_faces?: number[][];
  faces_vertices: number[][];
  faces_edges?: number[][];
  faceOrders?: Array<[number, number, number]>;
  [key: string]: unknown;
}

export interface CreaseParameter {
  face1: number;
  vertex1: number;
  face2: number;
  vertex2: number;
  edge: number;
  targetAngle: number;
}

export interface PreparedOrigamiModel {
  fold: FoldDocument;
  vertexCount: number;
  edgeCount: number;
  faceCount: number;
  positions: Float32Array;
  originalPositions: Float32Array;
  colors: Float32Array;
  indices: Uint32Array;
  edgesVertices: [number, number][];
  edgesAssignment: FoldAssignment[];
  edgesFoldAngle: Array<number | null>;
  facesVertices: number[][];
  facesEdges: number[][];
  edgesFaces: number[][];
  creaseParams: CreaseParameter[];
  diagnostics: SimulatorDiagnostics;
}

export interface PrepareFoldOptions {
  triangulate?: boolean;
  foldUseAngles?: boolean;
}

export interface SimulatorOptions {
  foldPercent?: number;
  axialStiffness?: number;
  creaseStiffness?: number;
  panelStiffness?: number;
  damping?: number;
  timeStep?: number;
  stepsPerFrame?: number;
  autoRender?: boolean;
}

export interface SimulatorDiagnostics {
  warnings: string[];
  errors: string[];
  webglAvailable?: boolean;
  usedCpuFallback?: boolean;
  maxEdgeStrain?: number;
  averageEdgeStrain?: number;
}

export interface SimulationFrame {
  positions: Float32Array;
  colors: Float32Array;
  indices: Uint32Array;
  diagnostics: SimulatorDiagnostics;
  step: number;
  foldPercent: number;
}

export interface CreateSimulatorConfig {
  model: PreparedOrigamiModel;
  canvas?: HTMLCanvasElement | OffscreenCanvas;
  gl?: WebGL2RenderingContext | WebGLRenderingContext;
  options?: SimulatorOptions;
}

export interface OrigamiSimulatorController {
  setFoldPercent(percent: number): void;
  setMaterial(options: Partial<SimulatorOptions>): void;
  step(numSteps?: number): SimulationFrame;
  start(): void;
  pause(): void;
  reset(): void;
  readFrame(): SimulationFrame;
  dispose(): void;
}
