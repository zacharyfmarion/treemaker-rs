import type { StateCreator } from 'zustand';
import type {
  ConditionKind,
  FoldArtifacts,
  OptimizationReport,
  TreeEdit,
  WasmErrorEnvelope,
} from '../../engine/types';
import type { Point } from '../../lib/geometry';
import type {
  AppStatus,
  CreaseColorMode,
  DocumentMode,
  Selection,
  ToolMode,
  TreeProject,
} from '../../lib/sampleProject';
import type {
  SymmetryAuthoringPair,
  SymmetryLeafPreview,
} from '../../lib/symmetryAuthoring';
import type { FileService } from '../../platform/fileService';
import type { ImportedCreasePatternDocument } from '../../lib/creasePatternImport';

export interface RecentProject {
  id: string;
  title: string;
  filename: string;
  savedAt: string;
  text: string;
}

export interface ProjectSliceState {
  project: TreeProject;
  documentMode: DocumentMode;
  importedCreasePattern: ImportedCreasePatternDocument | null;
  projectLoadId: number;
  currentFilePath: string | null;
  currentFileName: string;
  projectMessage: string | null;
  recentProjects: RecentProject[];
  status: AppStatus;
  dirty: boolean;
  engineReady: boolean;
  error: WasmErrorEnvelope | null;
  lastOptimization: OptimizationReport | null;
  designViewportFitRequestId: number;
}

export interface ProjectSliceActions {
  initEngine: () => Promise<void>;
  createNewProject: () => Promise<void>;
  loadStarterProject: () => Promise<void>;
  loadProjectText: (
    text: string,
    source?: { title?: string; filename?: string; path?: string | null; dirty?: boolean }
  ) => Promise<void>;
  loadCreasePatternText: (
    text: string,
    source: { filename: string; path?: string | null }
  ) => Promise<void>;
  openProject: (fileService?: FileService) => Promise<boolean>;
  saveProject: (fileService?: FileService) => Promise<boolean>;
  saveProjectAs: (fileService?: FileService) => Promise<boolean>;
  exportV4: (fileService?: FileService) => Promise<boolean>;
  exportFold: (fileService?: FileService) => Promise<boolean>;
  exportSvg: (fileService?: FileService) => Promise<boolean>;
  exportPng: (fileService?: FileService) => Promise<boolean>;
  loadExampleProject: (id: string) => Promise<void>;
  loadRecentProject: (id: string) => Promise<void>;
  autosaveProject: () => Promise<void>;
  clearProjectMessage: () => void;
}

export type ProjectSlice = ProjectSliceState & ProjectSliceActions;

export interface HistoryEntry {
  text: string;
  label: string;
  timestamp: string;
}

export interface HistorySliceState {
  historyPast: HistoryEntry[];
  historyFuture: HistoryEntry[];
  historyBusy: boolean;
}

export interface HistorySliceActions {
  beginHistoryCheckpoint: () => Promise<string | null>;
  commitHistoryCheckpoint: (beforeText: string | null, label?: string) => void;
  clearHistory: () => void;
  undo: () => Promise<void>;
  redo: () => Promise<void>;
}

export type HistorySlice = HistorySliceState & HistorySliceActions;

export interface EditingSliceState {
  selection: Selection;
  toolMode: ToolMode;
  symmetryAuthoringPairs: SymmetryAuthoringPair[];
}

export interface EditingSliceActions {
  addNodeAt: (loc: Point, connectTo?: number) => Promise<void>;
  addNodeWithSymmetry: (loc: Point, connectTo?: number) => Promise<void>;
  moveNode: (id: number, loc: Point) => Promise<void>;
  moveNodeWithSymmetry: (id: number, loc: Point) => Promise<void>;
  addEdge: (node1: number, node2: number) => Promise<void>;
  updateNodeLabel: (id: number, label: string) => Promise<void>;
  updateEdge: (
    id: number,
    update: { label?: string; length?: number; strain?: number; stiffness?: number }
  ) => Promise<void>;
  deleteSelection: () => Promise<void>;
  select: (selection: Selection) => void;
  selectAll: () => void;
  selectNone: () => void;
  selectPathBetweenSelectedNodes: () => void;
  setToolMode: (toolMode: ToolMode) => void;
}

export type EditingSlice = EditingSliceState & EditingSliceActions;

export interface ConditionSliceActions {
  updatePaper: (update: { width?: number; height?: number }) => Promise<void>;
  setSymmetry: (update: {
    hasSymmetry?: boolean;
    symLoc?: Point;
    symAngle?: number;
  }) => Promise<void>;
  addCondition: (kind: ConditionKind) => Promise<void>;
  previewSymmetryLeafPairs: (nodeIds?: number[]) => SymmetryLeafPreview;
  applySymmetryLeafPairs: (nodeIds?: number[]) => Promise<SymmetryLeafPreview>;
  deleteCondition: (id: number) => Promise<void>;
  clearConditions: () => Promise<void>;
}

export type ConditionSlice = ConditionSliceActions;

export interface ClipboardNode {
  sourceId: number;
  label: string;
  loc: Point;
}

export interface ClipboardEdge {
  sourceId: number;
  sourceNodes: [number, number];
  label: string;
  length: number;
  strain: number;
  stiffness: number;
}

export interface TreeClipboardPayload {
  nodes: ClipboardNode[];
  edges: ClipboardEdge[];
}

export interface ClipboardSliceState {
  clipboard: TreeClipboardPayload | null;
  clipboardPasteCount: number;
}

export interface ClipboardSliceActions {
  copySelection: () => void;
  cutSelection: () => Promise<void>;
  pasteClipboard: () => Promise<void>;
}

export type ClipboardSlice = ClipboardSliceState & ClipboardSliceActions;

export interface CreasePatternSliceState {
  creaseColorMode: CreaseColorMode;
  foldArtifacts: FoldArtifacts | null;
  foldArtifactError: string | null;
}

export interface CreasePatternSliceActions {
  optimizeScale: () => Promise<void>;
  optimizeEdges: () => Promise<void>;
  optimizeStrain: () => Promise<void>;
  buildCreasePattern: () => Promise<void>;
  refreshFoldArtifacts: () => Promise<FoldArtifacts | null>;
  setCreaseColorMode: (mode: CreaseColorMode) => void;
}

export type CreasePatternSlice = CreasePatternSliceState & CreasePatternSliceActions;

export type WorkspaceState =
  ProjectSlice &
  HistorySlice &
  EditingSlice &
  ClipboardSlice &
  ConditionSlice &
  CreasePatternSlice;

export type WorkspaceSliceCreator<T> = StateCreator<
  WorkspaceState,
  [['zustand/devtools', never]],
  [],
  T
>;

export type { TreeEdit };
