import { wrap, type Remote } from 'comlink';
import { create } from 'zustand';
import { projectFromSnapshot } from '../engine/snapshotMapper';
import type {
  OptimizationReport,
  TreeEdit,
  TreeSnapshot,
  WasmErrorEnvelope,
} from '../engine/types';
import type { Point } from '../lib/geometry';
import {
  createSampleProject,
  type AppStatus,
  type CreaseColorMode,
  type Selection,
  type ToolMode,
  type TreeProject,
} from '../lib/sampleProject';
import type { TreemakerWorkerApi } from '../workers/treemakerWorker';

type EngineClient = Remote<TreemakerWorkerApi>;

interface WorkspaceState {
  project: TreeProject;
  selection: Selection;
  toolMode: ToolMode;
  creaseColorMode: CreaseColorMode;
  status: AppStatus;
  dirty: boolean;
  engineReady: boolean;
  error: WasmErrorEnvelope | null;
  lastOptimization: OptimizationReport | null;
  initEngine: () => Promise<void>;
  createNewProject: () => Promise<void>;
  loadStarterProject: () => Promise<void>;
  addNodeAt: (loc: Point, connectTo?: number) => Promise<void>;
  moveNode: (id: number, loc: Point) => Promise<void>;
  addEdge: (node1: number, node2: number) => Promise<void>;
  updateNodeLabel: (id: number, label: string) => Promise<void>;
  updateEdge: (
    id: number,
    update: { label?: string; length?: number; strain?: number; stiffness?: number }
  ) => Promise<void>;
  deleteSelection: () => Promise<void>;
  optimizeScale: () => Promise<void>;
  buildCreasePattern: () => Promise<void>;
  select: (selection: Selection) => void;
  setToolMode: (toolMode: ToolMode) => void;
  setCreaseColorMode: (mode: CreaseColorMode) => void;
}

let worker: Worker | null = null;
let engine: EngineClient | null = null;
let handle: number | null = null;
let starterPromise: Promise<TreeSnapshot> | null = null;

function engineError(error: unknown): WasmErrorEnvelope {
  if (
    error &&
    typeof error === 'object' &&
    'code' in error &&
    'message' in error &&
    typeof (error as { code: unknown }).code === 'string'
  ) {
    return error as WasmErrorEnvelope;
  }
  return {
    code: 'engine',
    message: error instanceof Error ? error.message : String(error),
  };
}

async function getEngine(): Promise<EngineClient> {
  if (engine) return engine;
  worker = new Worker(new URL('../workers/treemakerWorker.ts', import.meta.url), {
    type: 'module',
  });
  engine = wrap<TreemakerWorkerApi>(worker);
  return engine;
}

async function replaceHandle(nextHandle: number) {
  if (engine && handle !== null) {
    await engine.freeTree(handle).catch(() => undefined);
  }
  handle = nextHandle;
}

async function createStarterTree(api: EngineClient): Promise<TreeSnapshot> {
  const nextHandle = await api.newDesign({ paper_width: 1, paper_height: 1 });
  try {
    await api.applyEdit(nextHandle, {
      type: 'add_node',
      loc: { x: 0.5, y: 0.46 },
      label: 'root',
    });
    for (const [x, y] of [
      [0.2, 0.2],
      [0.82, 0.22],
      [0.5, 0.82],
    ] as const) {
      await api.applyEdit(nextHandle, {
        type: 'add_node',
        loc: { x, y },
        connect_to: 1,
        edge_length: 1,
      });
    }
    const snapshot = await api.snapshot(nextHandle);
    await replaceHandle(nextHandle);
    return snapshot;
  } catch (error) {
    await api.freeTree(nextHandle).catch(() => undefined);
    throw error;
  }
}

async function createBlankTree(api: EngineClient): Promise<TreeSnapshot> {
  const nextHandle = await api.newDesign({ paper_width: 1, paper_height: 1 });
  try {
    const snapshot = await api.snapshot(nextHandle);
    await replaceHandle(nextHandle);
    return snapshot;
  } catch (error) {
    await api.freeTree(nextHandle).catch(() => undefined);
    throw error;
  }
}

async function initializeStarterTree(api: EngineClient): Promise<TreeSnapshot> {
  if (handle !== null) return api.snapshot(handle);
  starterPromise ??= createStarterTree(api).finally(() => {
    starterPromise = null;
  });
  return starterPromise;
}

async function requireHandle(): Promise<{ api: EngineClient; treeHandle: number }> {
  const api = await getEngine();
  if (handle === null) {
    const snapshot = await initializeStarterTree(api);
    useWorkspaceStore.setState({
      project: projectFromSnapshot(snapshot),
      engineReady: true,
      status: 'ready',
      error: null,
    });
  }
  if (handle === null) {
    throw new Error('TreeMaker engine did not create a tree handle');
  }
  return { api, treeHandle: handle };
}

function statusAfterEdit(snapshot: TreeSnapshot): AppStatus {
  return snapshot.edges.length > 0 ? 'needs_optimization' : 'ready';
}

function nextSelectionForEdit(
  edit: TreeEdit,
  snapshot: TreeSnapshot,
  createdNode?: number,
  createdEdge?: number
): Selection {
  if (createdNode !== undefined) return { kind: 'node', id: createdNode };
  if (createdEdge !== undefined) return { kind: 'edge', id: createdEdge };
  if ('id' in edit) {
    if (edit.type === 'move_node' || edit.type === 'update_node_label') {
      return { kind: 'node', id: edit.id };
    }
    if (edit.type === 'update_edge') return { kind: 'edge', id: edit.id };
  }
  if (snapshot.nodes.length > 0) return { kind: 'node', id: snapshot.nodes[0].id };
  return { kind: 'tree' };
}

export const useWorkspaceStore = create<WorkspaceState>((set, get) => ({
  project: createSampleProject(),
  selection: { kind: 'tree' },
  toolMode: 'select',
  creaseColorMode: 'mvf',
  status: 'loading_engine',
  dirty: false,
  engineReady: false,
  error: null,
  lastOptimization: null,

  initEngine: async () => {
    set({ status: 'loading_engine', error: null });
    try {
      const api = await getEngine();
      const snapshot = await initializeStarterTree(api);
      set({
        project: projectFromSnapshot(snapshot),
        selection: { kind: 'tree' },
        status: 'ready',
        dirty: false,
        engineReady: true,
        error: null,
        lastOptimization: null,
      });
    } catch (error) {
      set({ status: 'error', error: engineError(error), engineReady: false });
    }
  },

  createNewProject: async () => {
    set({ status: 'loading_engine', error: null });
    try {
      const api = await getEngine();
      const snapshot = await createBlankTree(api);
      set({
        project: projectFromSnapshot(snapshot),
        selection: { kind: 'tree' },
        toolMode: 'select',
        creaseColorMode: 'mvf',
        status: 'ready',
        dirty: false,
        engineReady: true,
        error: null,
        lastOptimization: null,
      });
    } catch (error) {
      set({ status: 'error', error: engineError(error) });
    }
  },

  loadStarterProject: async () => {
    set({ status: 'loading_engine', error: null });
    try {
      const api = await getEngine();
      const snapshot = await createStarterTree(api);
      set({
        project: projectFromSnapshot(snapshot),
        selection: { kind: 'tree' },
        toolMode: 'select',
        creaseColorMode: 'mvf',
        status: 'ready',
        dirty: false,
        engineReady: true,
        error: null,
        lastOptimization: null,
      });
    } catch (error) {
      set({ status: 'error', error: engineError(error) });
    }
  },

  addNodeAt: async (loc, connectTo) => {
    set({ error: null });
    try {
      const { api, treeHandle } = await requireHandle();
      const report = await api.applyEdit(treeHandle, {
        type: 'add_node',
        loc,
        connect_to: connectTo,
        edge_length: connectTo === undefined ? undefined : 1,
      });
      set({
        project: projectFromSnapshot(report.snapshot),
        selection: nextSelectionForEdit(
          { type: 'add_node', loc, connect_to: connectTo },
          report.snapshot,
          report.created_node,
          report.created_edge
        ),
        status: statusAfterEdit(report.snapshot),
        dirty: true,
        error: null,
        lastOptimization: null,
      });
    } catch (error) {
      set({ status: 'error', error: engineError(error) });
    }
  },

  moveNode: async (id, loc) => {
    set({ error: null });
    try {
      const { api, treeHandle } = await requireHandle();
      const edit: TreeEdit = { type: 'move_node', id, loc };
      const report = await api.applyEdit(treeHandle, edit);
      set({
        project: projectFromSnapshot(report.snapshot),
        selection: nextSelectionForEdit(edit, report.snapshot),
        status: statusAfterEdit(report.snapshot),
        dirty: true,
        error: null,
        lastOptimization: null,
      });
    } catch (error) {
      set({ status: 'error', error: engineError(error) });
    }
  },

  addEdge: async (node1, node2) => {
    if (node1 === node2) return;
    set({ error: null });
    try {
      const { api, treeHandle } = await requireHandle();
      const report = await api.applyEdit(treeHandle, {
        type: 'add_edge',
        node1,
        node2,
        length: 1,
      });
      set({
        project: projectFromSnapshot(report.snapshot),
        selection: report.created_edge
          ? { kind: 'edge', id: report.created_edge }
          : { kind: 'node', id: node2 },
        status: statusAfterEdit(report.snapshot),
        dirty: true,
        error: null,
        lastOptimization: null,
      });
    } catch (error) {
      set({ status: 'error', error: engineError(error) });
    }
  },

  updateNodeLabel: async (id, label) => {
    set({ error: null });
    try {
      const { api, treeHandle } = await requireHandle();
      const edit: TreeEdit = { type: 'update_node_label', id, label };
      const report = await api.applyEdit(treeHandle, edit);
      set({
        project: projectFromSnapshot(report.snapshot),
        selection: nextSelectionForEdit(edit, report.snapshot),
        dirty: true,
        error: null,
      });
    } catch (error) {
      set({ status: 'error', error: engineError(error) });
    }
  },

  updateEdge: async (id, update) => {
    set({ error: null });
    try {
      const { api, treeHandle } = await requireHandle();
      const edit: TreeEdit = { type: 'update_edge', id, ...update };
      const report = await api.applyEdit(treeHandle, edit);
      set({
        project: projectFromSnapshot(report.snapshot),
        selection: nextSelectionForEdit(edit, report.snapshot),
        status: statusAfterEdit(report.snapshot),
        dirty: true,
        error: null,
        lastOptimization: null,
      });
    } catch (error) {
      set({ status: 'error', error: engineError(error) });
    }
  },

  deleteSelection: async () => {
    const selection = get().selection;
    if (selection.kind !== 'node' && selection.kind !== 'edge') return;
    set({ error: null });
    try {
      const { api, treeHandle } = await requireHandle();
      const edit: TreeEdit =
        selection.kind === 'node'
          ? { type: 'delete_node', id: selection.id }
          : { type: 'delete_edge', id: selection.id };
      const report = await api.applyEdit(treeHandle, edit);
      set({
        project: projectFromSnapshot(report.snapshot),
        selection: { kind: 'tree' },
        status: statusAfterEdit(report.snapshot),
        dirty: true,
        error: null,
        lastOptimization: null,
      });
    } catch (error) {
      set({ status: 'error', error: engineError(error) });
    }
  },

  optimizeScale: async () => {
    set({ status: 'optimizing', error: null });
    try {
      const { api, treeHandle } = await requireHandle();
      const report = await api.optimizeScale(treeHandle);
      const snapshot = await api.snapshot(treeHandle);
      set({
        project: projectFromSnapshot(snapshot),
        status: report.is_feasible ? 'optimized' : 'needs_optimization',
        error: null,
        lastOptimization: report,
        dirty: true,
      });
    } catch (error) {
      set({ status: 'error', error: engineError(error) });
    }
  },

  buildCreasePattern: async () => {
    set({ status: 'building_crease_pattern', error: null });
    try {
      const { api, treeHandle } = await requireHandle();
      const snapshot = await api.buildCreasePattern(treeHandle);
      set({
        project: projectFromSnapshot(snapshot),
        status: 'crease_pattern_ready',
        error: null,
        dirty: true,
      });
    } catch (error) {
      set({ status: 'error', error: engineError(error) });
    }
  },

  select: (selection) => set({ selection }),
  setToolMode: (toolMode) => set({ toolMode }),
  setCreaseColorMode: (creaseColorMode) => set({ creaseColorMode }),
}));
