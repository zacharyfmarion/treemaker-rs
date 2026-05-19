import { projectFromSnapshot } from '../../../engine/snapshotMapper';
import type { FoldArtifacts, OptimizationReport } from '../../../engine/types';
import { DEFAULT_CREASE_COLOR_MODE } from '../../../lib/sampleProject';
import { useLayoutStore } from '../../layoutStore';
import { selectWorkspaceCapabilities } from '../capabilities';
import {
  engineError,
  ensureTreeHandle,
  projectStateFromSnapshot,
  type EngineClient,
} from '../engineRuntime';
import type { CreasePatternSlice, WorkspaceSliceCreator } from '../types';
import type { WorkspaceCapabilityId } from '../../../lib/workspaceCapabilities';

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

  async function loadFoldArtifacts(): Promise<FoldArtifacts | null> {
    if (get().documentMode === 'crease-pattern') return get().foldArtifacts;
    try {
      const { api, treeHandle } = await requireActiveTree();
      const foldArtifacts = await api.foldArtifacts(treeHandle);
      set({ foldArtifacts, foldArtifactError: null });
      return foldArtifacts;
    } catch (error) {
      set({ foldArtifacts: null, foldArtifactError: engineError(error).message });
      return null;
    }
  }

  async function runOptimization(
    label: string,
    capabilityId: WorkspaceCapabilityId,
    optimize: (api: EngineClient, treeHandle: number) => Promise<OptimizationReport>,
    options: { fitPaperView?: boolean } = {}
  ) {
    const capability = selectWorkspaceCapabilities(get())[capabilityId];
    if (!capability.enabled) {
      set({ error: { code: 'invalid_operation', message: capability.reason } });
      return;
    }
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
        foldArtifacts: null,
        foldArtifactError: null,
        dirty: true,
        projectMessage: label,
        designViewportFitRequestId: options.fitPaperView
          ? get().designViewportFitRequestId + 1
          : get().designViewportFitRequestId,
      });
      get().commitHistoryCheckpoint(checkpoint, label);
      void get().autosaveProject();
    } catch (error) {
      set({ status: 'error', error: engineError(error) });
    }
  }

  return {
    creaseColorMode: DEFAULT_CREASE_COLOR_MODE,
    foldArtifacts: null,
    foldArtifactError: null,

    optimizeScale: async () => {
      await runOptimization('Optimize scale', 'optimize.scale', (api, treeHandle) =>
        api.optimizeScale(treeHandle),
        { fitPaperView: true }
      );
    },

    optimizeEdges: async () => {
      await runOptimization('Optimize edges', 'optimize.edges', (api, treeHandle) =>
        api.optimizeEdges(treeHandle)
      );
    },

    optimizeStrain: async () => {
      await runOptimization('Optimize strain', 'optimize.strain', (api, treeHandle) =>
        api.optimizeStrain(treeHandle)
      );
    },

    buildCreasePattern: async () => {
      const capability = selectWorkspaceCapabilities(get())['cp.build'];
      if (!capability.enabled) {
        set({
          error: {
            code: 'invalid_operation',
            message: capability.reason,
          },
        });
        return;
      }

      set({ status: 'building_crease_pattern', error: null });
      const checkpoint = await get().beginHistoryCheckpoint();
      try {
        const { api, treeHandle } = await requireActiveTree();
        const snapshot = await api.buildCreasePattern(treeHandle);
        let foldArtifactError: string | null = null;
        const foldArtifacts = await api.foldArtifacts(treeHandle).catch((error) => {
          foldArtifactError = engineError(error).message;
          return null;
        });
        set({
          project: projectFromSnapshot(snapshot, get().project.title),
          status: 'crease_pattern_ready',
          error: null,
          foldArtifacts,
          foldArtifactError,
          dirty: true,
          projectMessage: 'Built crease pattern',
        });
        get().commitHistoryCheckpoint(checkpoint, 'Build crease pattern');
        void get().autosaveProject();
        useLayoutStore.getState().activatePanel('crease-pattern');
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
      }
    },

    refreshFoldArtifacts: loadFoldArtifacts,

    setCreaseColorMode: (creaseColorMode) => set({ creaseColorMode }),
  };
};
