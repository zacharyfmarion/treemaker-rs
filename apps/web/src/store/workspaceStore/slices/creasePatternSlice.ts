import { projectFromSnapshot } from '../../../engine/snapshotMapper';
import type { FoldArtifacts, OptimizationReport } from '../../../engine/types';
import { DEFAULT_CREASE_COLOR_MODE } from '../../../lib/sampleProject';
import { useLayoutStore } from '../../layoutStore';
import { selectWorkspaceCapabilities } from '../capabilities';
import {
  engineError,
  ensureTreeHandle,
  getEngine,
  projectStateFromSnapshot,
  type EngineClient,
} from '../engineRuntime';
import type { CreasePatternSlice, WorkspaceSliceCreator } from '../types';
import type { WorkspaceCapabilityId } from '../../../lib/workspaceCapabilities';

export const createCreasePatternSlice: WorkspaceSliceCreator<CreasePatternSlice> = (
  set,
  get
) => {
  const wholeSimulationFocus = { kind: 'whole' as const };

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
      set({
        foldArtifacts,
        foldArtifactError: null,
        sequenceTarget: null,
        sequencePlan: null,
        sequenceSimulationFocus: wholeSimulationFocus,
      });
      return foldArtifacts;
    } catch (error) {
      set({
        foldArtifacts: null,
        foldArtifactError: engineError(error).message,
        sequenceTarget: null,
        sequencePlan: null,
        sequenceSimulationFocus: wholeSimulationFocus,
      });
      return null;
    }
  }

  async function requireFoldForSequence(): Promise<FoldArtifacts | null> {
    const foldArtifacts = get().foldArtifacts ?? (await loadFoldArtifacts());
    if (!foldArtifacts) {
      set({
        sequencePlanning: false,
        sequenceError: 'No crease pattern is available for sequence planning.',
      });
      return null;
    }
    return foldArtifacts;
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
        sequenceTarget: null,
        sequencePlan: null,
        sequenceSimulationFocus: wholeSimulationFocus,
        sequenceError: null,
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
    sequenceTarget: null,
    sequencePlan: null,
    sequenceSimulationFocus: wholeSimulationFocus,
    sequencePlanning: false,
    sequenceError: null,

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
        const project = projectFromSnapshot(snapshot, get().project.title);
        const hasDrawableCreasePattern = project.creases.length > 0 || project.facets.length > 0;

        if (!hasDrawableCreasePattern) {
          set({
            project,
            status:
              project.edges.length === 0
                ? 'ready'
                : snapshot.summary.is_feasible
                  ? 'optimized'
                  : 'needs_optimization',
            error: {
              code: 'invalid_operation',
              message: 'Build CP completed but did not produce drawable crease-pattern geometry.',
            },
            foldArtifacts: null,
            foldArtifactError: null,
            sequenceTarget: null,
            sequencePlan: null,
            sequenceSimulationFocus: wholeSimulationFocus,
            sequenceError: null,
            projectMessage: null,
          });
          return;
        }

        let foldArtifactError: string | null = null;
        const foldArtifacts = await api.foldArtifacts(treeHandle).catch((error) => {
          foldArtifactError = engineError(error).message;
          return null;
        });
        set({
          project,
          status: 'crease_pattern_ready',
          error: null,
          foldArtifacts,
          foldArtifactError,
          sequenceTarget: null,
          sequencePlan: null,
          sequenceSimulationFocus: wholeSimulationFocus,
          sequenceError: null,
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

    analyzeSequenceTarget: async () => {
      set({ sequencePlanning: true, sequenceError: null });
      try {
        const foldArtifacts = await requireFoldForSequence();
        if (!foldArtifacts) return null;
        const api = await getEngine();
        const target = await api.sequenceAnalyzeFold(JSON.stringify(foldArtifacts.fold), {
          solution_limit: 10,
        });
        set({ sequenceTarget: target, sequencePlanning: false, sequenceError: null });
        return target;
      } catch (error) {
        const message = engineError(error).message;
        set({ sequencePlanning: false, sequenceError: message, sequenceTarget: null });
        return null;
      }
    },

    planFoldingSequence: async () => {
      set({ sequencePlanning: true, sequenceError: null });
      try {
        const foldArtifacts = await requireFoldForSequence();
        if (!foldArtifacts) return null;
        const api = await getEngine();
        const foldJson = JSON.stringify(foldArtifacts.fold);
        const [target, plan] = await Promise.all([
          api.sequenceAnalyzeFold(foldJson, { solution_limit: 10 }),
          api.sequencePlanFold(foldJson, {
            solution_limit: 10,
            max_steps: 64,
            max_states: 1024,
          }),
        ]);
        set({
          sequenceTarget: target,
          sequencePlan: plan,
          sequenceSimulationFocus: wholeSimulationFocus,
          sequencePlanning: false,
          sequenceError: null,
        });
        useLayoutStore.getState().activatePanel('sequence');
        return plan;
      } catch (error) {
        const message = engineError(error).message;
        set({
          sequencePlanning: false,
          sequenceError: message,
          sequencePlan: null,
          sequenceSimulationFocus: wholeSimulationFocus,
        });
        return null;
      }
    },

    setCreaseColorMode: (creaseColorMode) => set({ creaseColorMode }),
    setSequenceSimulationFocus: (sequenceSimulationFocus) => set({ sequenceSimulationFocus }),
  };
};
