import { projectFromSnapshot } from '../../../engine/snapshotMapper';
import type { FoldArtifacts, OptimizationReport } from '../../../engine/types';
import { DEFAULT_CREASE_COLOR_MODE } from '../../../lib/sampleProject';
import { getCreasePatternWorkflowState } from '../../../lib/workflowAvailability';
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

  async function loadFoldArtifacts(): Promise<FoldArtifacts | null> {
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
    optimize: (api: EngineClient, treeHandle: number) => Promise<OptimizationReport>,
    options: { fitPaperView?: boolean } = {}
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
      await runOptimization('Optimize scale', (api, treeHandle) => api.optimizeScale(treeHandle), {
        fitPaperView: true,
      });
    },

    optimizeEdges: async () => {
      await runOptimization('Optimize edges', (api, treeHandle) => api.optimizeEdges(treeHandle));
    },

    optimizeStrain: async () => {
      await runOptimization('Optimize strain', (api, treeHandle) => api.optimizeStrain(treeHandle));
    },

    buildCreasePattern: async () => {
      const workflowState = getCreasePatternWorkflowState({
        engineReady: get().engineReady,
        status: get().status,
        edgeCount: get().project.edges.length,
      });
      if (!workflowState.canBuildCreasePattern) {
        set({
          error: {
            code: 'invalid_operation',
            message: workflowState.buildCreasePatternReason,
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
