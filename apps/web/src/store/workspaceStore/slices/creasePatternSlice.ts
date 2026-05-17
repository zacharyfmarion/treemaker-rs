import { projectFromSnapshot } from '../../../engine/snapshotMapper';
import type { OptimizationReport } from '../../../engine/types';
import { useLayoutStore } from '../../layoutStore';
import {
  engineError,
  ensureTreeHandle,
  projectStateFromSnapshot,
  type EngineClient,
} from '../engineRuntime';
import type { CreasePatternSlice, WorkspaceSliceCreator } from '../types';

export const createCreasePatternSlice: WorkspaceSliceCreator<CreasePatternSlice> = (
  set,
  get
) => {
  async function requireActiveTree() {
    const result = await ensureTreeHandle();
    if (result.initializedSnapshot) {
      set(projectStateFromSnapshot(result.initializedSnapshot, get().project.title));
    }
    return result;
  }

  async function runOptimization(
    label: string,
    optimize: (api: EngineClient, treeHandle: number) => Promise<OptimizationReport>
  ) {
    set({ status: 'optimizing', error: null });
    const checkpoint = await get().beginHistoryCheckpoint();
    try {
      const { api, treeHandle } = await requireActiveTree();
      const report = await optimize(api, treeHandle);
      const snapshot = await api.snapshot(treeHandle);
      set({
        project: projectFromSnapshot(snapshot, get().project.title),
        status: report.is_feasible ? 'optimized' : 'needs_optimization',
        error: null,
        lastOptimization: report,
        dirty: true,
        projectMessage: label,
      });
      get().commitHistoryCheckpoint(checkpoint, label);
      void get().autosaveProject();
    } catch (error) {
      set({ status: 'error', error: engineError(error) });
    }
  }

  return {
    creaseColorMode: 'mvf',

    optimizeScale: async () => {
      await runOptimization('Optimize scale', (api, treeHandle) => api.optimizeScale(treeHandle));
    },

    optimizeEdges: async () => {
      await runOptimization('Optimize edges', (api, treeHandle) => api.optimizeEdges(treeHandle));
    },

    optimizeStrain: async () => {
      await runOptimization('Optimize strain', (api, treeHandle) => api.optimizeStrain(treeHandle));
    },

    buildCreasePattern: async () => {
      set({ status: 'building_crease_pattern', error: null });
      const checkpoint = await get().beginHistoryCheckpoint();
      try {
        const { api, treeHandle } = await requireActiveTree();
        const snapshot = await api.buildCreasePattern(treeHandle);
        set({
          project: projectFromSnapshot(snapshot, get().project.title),
          status: 'crease_pattern_ready',
          error: null,
          dirty: true,
        });
        get().commitHistoryCheckpoint(checkpoint, 'Build crease pattern');
        void get().autosaveProject();
        useLayoutStore.getState().activatePanel('crease-pattern');
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
      }
    },

    setCreaseColorMode: (creaseColorMode) => set({ creaseColorMode }),
  };
};
