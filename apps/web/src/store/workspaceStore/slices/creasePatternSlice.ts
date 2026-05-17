import { projectFromSnapshot } from '../../../engine/snapshotMapper';
import { useLayoutStore } from '../../layoutStore';
import { engineError, ensureTreeHandle, projectStateFromSnapshot } from '../engineRuntime';
import type { CreasePatternSlice, WorkspaceSliceCreator } from '../types';

export const createCreasePatternSlice: WorkspaceSliceCreator<CreasePatternSlice> = (
  set,
  _get
) => {
  async function requireActiveTree() {
    const result = await ensureTreeHandle();
    if (result.initializedSnapshot) {
      set(projectStateFromSnapshot(result.initializedSnapshot));
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
        const { api, treeHandle } = await requireActiveTree();
        const snapshot = await api.buildCreasePattern(treeHandle);
        set({
          project: projectFromSnapshot(snapshot),
          status: 'crease_pattern_ready',
          error: null,
          dirty: true,
        });
        useLayoutStore.getState().activatePanel('crease-pattern');
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
      }
    },

    setCreaseColorMode: (creaseColorMode) => set({ creaseColorMode }),
  };
};
