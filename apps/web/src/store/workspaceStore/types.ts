import type { StateCreator } from 'zustand';
import type {
  OptimizationReport,
  TreeEdit,
  WasmErrorEnvelope,
} from '../../engine/types';
import type { Point } from '../../lib/geometry';
import type {
  AppStatus,
  CreaseColorMode,
  Selection,
  ToolMode,
  TreeProject,
} from '../../lib/sampleProject';

export interface ProjectSliceState {
  project: TreeProject;
  status: AppStatus;
  dirty: boolean;
  engineReady: boolean;
  error: WasmErrorEnvelope | null;
  lastOptimization: OptimizationReport | null;
}

export interface ProjectSliceActions {
  initEngine: () => Promise<void>;
  createNewProject: () => Promise<void>;
  loadStarterProject: () => Promise<void>;
}

export type ProjectSlice = ProjectSliceState & ProjectSliceActions;

export interface EditingSliceState {
  selection: Selection;
  toolMode: ToolMode;
}

export interface EditingSliceActions {
  addNodeAt: (loc: Point, connectTo?: number) => Promise<void>;
  moveNode: (id: number, loc: Point) => Promise<void>;
  addEdge: (node1: number, node2: number) => Promise<void>;
  updateNodeLabel: (id: number, label: string) => Promise<void>;
  updateEdge: (
    id: number,
    update: { label?: string; length?: number; strain?: number; stiffness?: number }
  ) => Promise<void>;
  deleteSelection: () => Promise<void>;
  select: (selection: Selection) => void;
  setToolMode: (toolMode: ToolMode) => void;
}

export type EditingSlice = EditingSliceState & EditingSliceActions;

export interface CreasePatternSliceState {
  creaseColorMode: CreaseColorMode;
}

export interface CreasePatternSliceActions {
  optimizeScale: () => Promise<void>;
  buildCreasePattern: () => Promise<void>;
  setCreaseColorMode: (mode: CreaseColorMode) => void;
}

export type CreasePatternSlice = CreasePatternSliceState & CreasePatternSliceActions;

export type WorkspaceState = ProjectSlice & EditingSlice & CreasePatternSlice;

export type WorkspaceSliceCreator<T> = StateCreator<
  WorkspaceState,
  [['zustand/devtools', never]],
  [],
  T
>;

export type { TreeEdit };
