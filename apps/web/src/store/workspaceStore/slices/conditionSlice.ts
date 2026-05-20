import type { ConditionKind, TreeSnapshot } from '../../../engine/types';
import { projectFromSnapshot } from '../../../engine/snapshotMapper';
import {
  engineError,
  ensureTreeHandle,
  projectStateFromSnapshot,
  statusAfterEdit,
} from '../engineRuntime';
import type { ConditionSlice, WorkspaceSliceCreator } from '../types';

export const createConditionSlice: WorkspaceSliceCreator<ConditionSlice> = (set, get) => {
  function rejectReadOnly() {
    if (get().documentMode === 'tree') return false;
    set({
      error: {
        code: 'invalid_operation',
        message: 'Conditions require an editable tree document',
      },
    });
    return true;
  }

  async function applyConditionEdit(
    edit:
      | { type: 'update_paper'; width: number; height: number }
      | {
          type: 'set_symmetry';
          has_symmetry: boolean;
          sym_loc?: { x: number; y: number };
          sym_angle?: number;
        }
      | { type: 'add_condition'; kind: ConditionKind }
      | { type: 'delete_condition'; id: number },
    label: string
  ) {
    if (rejectReadOnly()) return;
    set({ error: null });
    const checkpoint = await get().beginHistoryCheckpoint();
    try {
      const { api, treeHandle, initializedSnapshot } = await ensureTreeHandle();
      if (initializedSnapshot) {
        set(projectStateFromSnapshot(initializedSnapshot, get().project.title));
      }
      const report = await api.applyEdit(treeHandle, edit);
      set({
        project: projectFromSnapshot(report.snapshot, get().project.title),
        status: statusAfterEdit(report.snapshot),
        dirty: true,
        error: null,
        lastOptimization: null,
        foldArtifacts: null,
        foldArtifactError: null,
        projectMessage: label,
      });
      get().commitHistoryCheckpoint(checkpoint, label);
      void get().autosaveProject();
    } catch (error) {
      set({ status: 'error', error: engineError(error) });
    }
  }

  return {
    updatePaper: async (update) => {
      const width = update.width ?? get().project.paper.width;
      const height = update.height ?? get().project.paper.height;
      await applyConditionEdit({ type: 'update_paper', width, height }, 'Updated paper');
    },

    setSymmetry: async (update) => {
      const project = get().project;
      await applyConditionEdit(
        {
          type: 'set_symmetry',
          has_symmetry: update.hasSymmetry ?? project.hasSymmetry,
          sym_loc: update.symLoc ?? project.paper.symLoc,
          sym_angle: update.symAngle ?? project.paper.symAngle,
        },
        'Updated symmetry'
      );
    },

    addCondition: async (kind) => {
      await applyConditionEdit({ type: 'add_condition', kind }, 'Added condition');
    },

    deleteCondition: async (id) => {
      await applyConditionEdit({ type: 'delete_condition', id }, 'Removed condition');
    },

    clearConditions: async () => {
      if (rejectReadOnly()) return;
      const ids = get()
        .project.conditions.map((condition) => condition.id)
        .sort((a, b) => b - a);
      if (ids.length === 0) return;
      set({ error: null });
      const checkpoint = await get().beginHistoryCheckpoint();
      try {
        const { api, treeHandle } = await ensureTreeHandle();
        let snapshot: TreeSnapshot | null = null;
        for (const id of ids) {
          const report = await api.applyEdit(treeHandle, { type: 'delete_condition', id });
          snapshot = report.snapshot;
        }
        if (!snapshot) return;
        set({
          project: projectFromSnapshot(snapshot, get().project.title),
          selection: { kind: 'tree' },
          status: statusAfterEdit(snapshot),
          dirty: true,
          error: null,
          lastOptimization: null,
          foldArtifacts: null,
          foldArtifactError: null,
          projectMessage: 'Cleared conditions',
        });
        get().commitHistoryCheckpoint(checkpoint, 'Clear conditions');
        void get().autosaveProject();
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
      }
    },
  };
};
