import { projectFromSnapshot } from '../../../engine/snapshotMapper';
import { useLayoutStore } from '../../layoutStore';
import { engineError, ensureTreeHandle, projectStateFromSnapshot } from '../engineRuntime';
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

  return {
    creaseColorMode: 'mvf',

    optimizeScale: async () => {
      set({ status: 'optimizing', error: null });
      try {
        const { api, treeHandle } = await requireActiveTree();
        const report = await api.optimizeScale(treeHandle);
        const snapshot = await api.snapshot(treeHandle);
        set({
          project: projectFromSnapshot(snapshot, get().project.title),
          status: report.is_feasible ? 'optimized' : 'needs_optimization',
          error: null,
          lastOptimization: report,
          dirty: true,
        });
        void get().autosaveProject();
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
      }
    },

    buildCreasePattern: async () => {
      set({ status: 'building_crease_pattern', error: null });
      try {
        const { api, treeHandle } = await requireActiveTree();
        const snapshot = await api.buildCreasePattern(treeHandle);
        set({
          project: projectFromSnapshot(snapshot, get().project.title),
          status: 'crease_pattern_ready',
          error: null,
          dirty: true,
        });
        void get().autosaveProject();
        useLayoutStore.getState().activatePanel('crease-pattern');
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
      }
    },

    setCreaseColorMode: (creaseColorMode) => set({ creaseColorMode }),
  };
};
