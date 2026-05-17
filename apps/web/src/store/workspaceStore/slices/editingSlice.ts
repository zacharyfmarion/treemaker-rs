import type { TreeEdit, TreeSnapshot } from '../../../engine/types';
import { projectFromSnapshot } from '../../../engine/snapshotMapper';
import {
  selectEverything,
  selectedEdgeIds,
  selectedNodeIds,
} from '../../../lib/selection';
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
      set(projectStateFromSnapshot(result.initializedSnapshot, get().project.title));
    }
    return result;
  }

  return {
    selection: { kind: 'tree' },
    toolMode: 'select',

    addNodeAt: async (loc, connectTo) => {
      set({ error: null });
      const checkpoint = await get().beginHistoryCheckpoint();
      try {
        const { api, treeHandle } = await requireActiveTree();
        const report = await api.applyEdit(treeHandle, {
          type: 'add_node',
          loc,
          connect_to: connectTo,
          edge_length: connectTo === undefined ? undefined : 1,
        });
        set({
          project: projectFromSnapshot(report.snapshot, get().project.title),
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
        get().commitHistoryCheckpoint(checkpoint, 'Add node');
        void get().autosaveProject();
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
      }
    },

    moveNode: async (id, loc) => {
      set({ error: null });
      const checkpoint = await get().beginHistoryCheckpoint();
      try {
        const { api, treeHandle } = await requireActiveTree();
        const edit: TreeEdit = { type: 'move_node', id, loc };
        const report = await api.applyEdit(treeHandle, edit);
        set({
          project: projectFromSnapshot(report.snapshot, get().project.title),
          selection: nextSelectionForEdit(edit, report.snapshot),
          status: statusAfterEdit(report.snapshot),
          dirty: true,
          error: null,
          lastOptimization: null,
        });
        get().commitHistoryCheckpoint(checkpoint, 'Move node');
        void get().autosaveProject();
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
      }
    },

    addEdge: async (node1, node2) => {
      if (node1 === node2) return;
      set({ error: null });
      const checkpoint = await get().beginHistoryCheckpoint();
      try {
        const { api, treeHandle } = await requireActiveTree();
        const report = await api.applyEdit(treeHandle, {
          type: 'add_edge',
          node1,
          node2,
          length: 1,
        });
        set({
          project: projectFromSnapshot(report.snapshot, get().project.title),
          selection: report.created_edge
            ? { kind: 'edge', id: report.created_edge }
            : { kind: 'node', id: node2 },
          status: statusAfterEdit(report.snapshot),
          dirty: true,
          error: null,
          lastOptimization: null,
        });
        get().commitHistoryCheckpoint(checkpoint, 'Add edge');
        void get().autosaveProject();
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
      }
    },

    updateNodeLabel: async (id, label) => {
      set({ error: null });
      const checkpoint = await get().beginHistoryCheckpoint();
      try {
        const { api, treeHandle } = await requireActiveTree();
        const edit: TreeEdit = { type: 'update_node_label', id, label };
        const report = await api.applyEdit(treeHandle, edit);
        set({
          project: projectFromSnapshot(report.snapshot, get().project.title),
          selection: nextSelectionForEdit(edit, report.snapshot),
          dirty: true,
          error: null,
        });
        get().commitHistoryCheckpoint(checkpoint, 'Rename node');
        void get().autosaveProject();
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
      }
    },

    updateEdge: async (id, update) => {
      set({ error: null });
      const checkpoint = await get().beginHistoryCheckpoint();
      try {
        const { api, treeHandle } = await requireActiveTree();
        const edit: TreeEdit = { type: 'update_edge', id, ...update };
        const report = await api.applyEdit(treeHandle, edit);
        set({
          project: projectFromSnapshot(report.snapshot, get().project.title),
          selection: nextSelectionForEdit(edit, report.snapshot),
          status: statusAfterEdit(report.snapshot),
          dirty: true,
          error: null,
          lastOptimization: null,
        });
        get().commitHistoryCheckpoint(checkpoint, 'Edit edge');
        void get().autosaveProject();
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
      }
    },

    deleteSelection: async () => {
      const selection = get().selection;
      const nodeIds = selectedNodeIds(selection).sort((a, b) => b - a);
      const edgeIds = selectedEdgeIds(selection).sort((a, b) => b - a);
      if (nodeIds.length === 0 && edgeIds.length === 0) return;
      set({ error: null });
      const checkpoint = await get().beginHistoryCheckpoint();
      try {
        const { api, treeHandle } = await requireActiveTree();
        let snapshot: TreeSnapshot | null = null;
        for (const id of edgeIds) {
          const report = await api.applyEdit(treeHandle, { type: 'delete_edge', id });
          snapshot = report.snapshot;
        }
        for (const id of nodeIds) {
          const report = await api.applyEdit(treeHandle, { type: 'delete_node', id });
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
        });
        get().commitHistoryCheckpoint(checkpoint, 'Delete selection');
        void get().autosaveProject();
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
      }
    },

    select: (selection) => set({ selection }),
    selectAll: () => set({ selection: selectEverything(get().project) }),
    selectNone: () => set({ selection: { kind: 'tree' } }),
    selectPathBetweenSelectedNodes: () => {
      const [a, b] = selectedNodeIds(get().selection);
      if (a === undefined || b === undefined) return;
      const path = get().project.paths.find(
        (candidate) =>
          (candidate.nodes[0] === a && candidate.nodes[1] === b) ||
          (candidate.nodes[0] === b && candidate.nodes[1] === a)
      );
      if (path) set({ selection: { kind: 'path', id: path.id } });
    },
    setToolMode: (toolMode) => set({ toolMode }),
  };
};
