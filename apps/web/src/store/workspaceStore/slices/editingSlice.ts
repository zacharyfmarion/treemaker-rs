import type { TreeEdit } from '../../../engine/types';
import { projectFromSnapshot } from '../../../engine/snapshotMapper';
import {
  engineError,
  ensureTreeHandle,
  nextSelectionForEdit,
  projectStateFromSnapshot,
  statusAfterEdit,
} from '../engineRuntime';
import type { EditingSlice, WorkspaceSliceCreator } from '../types';

export const createEditingSlice: WorkspaceSliceCreator<EditingSlice> = (set, get) => {
  async function requireActiveTree() {
    const result = await ensureTreeHandle();
    if (result.initializedSnapshot) {
      set(projectStateFromSnapshot(result.initializedSnapshot));
    }
    return result;
  }

  return {
    selection: { kind: 'tree' },
    toolMode: 'select',

    addNodeAt: async (loc, connectTo) => {
      set({ error: null });
      try {
        const { api, treeHandle } = await requireActiveTree();
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
        const { api, treeHandle } = await requireActiveTree();
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
        const { api, treeHandle } = await requireActiveTree();
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
        const { api, treeHandle } = await requireActiveTree();
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
        const { api, treeHandle } = await requireActiveTree();
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
        const { api, treeHandle } = await requireActiveTree();
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

    select: (selection) => set({ selection }),
    setToolMode: (toolMode) => set({ toolMode }),
  };
};
