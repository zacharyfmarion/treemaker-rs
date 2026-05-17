import { expose } from 'comlink';
import init, {
  apply_edit,
  build_crease_pattern,
  free_tree,
  new_design,
  optimize_edges,
  optimize_scale,
  optimize_strain,
  tree_snapshot,
} from '../generated/treemaker-wasm/treemaker_wasm';
import type {
  EditReport,
  OptimizationReport,
  TreeEdit,
  TreeSnapshot,
  WasmErrorEnvelope,
} from '../engine/types';

let ready: Promise<void> | null = null;

async function ensureReady() {
  ready ??= init().then(() => undefined);
  await ready;
}

function normalizeError(error: unknown): WasmErrorEnvelope {
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
    code: 'wasm_error',
    message: error instanceof Error ? error.message : String(error),
  };
}

async function call<T>(fn: () => T): Promise<T> {
  await ensureReady();
  try {
    return fn();
  } catch (error) {
    throw normalizeError(error);
  }
}

const api = {
  async newDesign(config?: { paper_width?: number; paper_height?: number }): Promise<number> {
    return call(() => new_design(config ?? null));
  },
  async snapshot(handle: number): Promise<TreeSnapshot> {
    return call(() => tree_snapshot(handle) as TreeSnapshot);
  },
  async applyEdit(handle: number, edit: TreeEdit): Promise<EditReport> {
    return call(() => apply_edit(handle, edit) as EditReport);
  },
  async optimizeScale(handle: number): Promise<OptimizationReport> {
    return call(() => optimize_scale(handle) as OptimizationReport);
  },
  async optimizeEdges(handle: number): Promise<OptimizationReport> {
    return call(() => optimize_edges(handle) as OptimizationReport);
  },
  async optimizeStrain(handle: number): Promise<OptimizationReport> {
    return call(() => optimize_strain(handle) as OptimizationReport);
  },
  async buildCreasePattern(handle: number): Promise<TreeSnapshot> {
    return call(() => {
      build_crease_pattern(handle);
      return tree_snapshot(handle) as TreeSnapshot;
    });
  },
  async freeTree(handle: number): Promise<void> {
    return call(() => free_tree(handle));
  },
};

export type TreemakerWorkerApi = typeof api;

expose(api);
