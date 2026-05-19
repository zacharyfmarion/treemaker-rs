import type { TreeEdit, TreeSnapshot } from '../../../engine/types';
import { projectFromSnapshot } from '../../../engine/snapshotMapper';
import {
  selectEverything,
  selectedEdgeIds,
  selectedNodeIds,
  selectionCoversAllNodes,
} from '../../../lib/selection';
import {
  addSymmetryAuthoringPair,
  filterSymmetryAuthoringPairs,
  findMirrorEdgeId,
  findMirrorNodeId,
  reflectPointAcrossSymmetryAxis,
  snapPointToSymmetryAxis,
  symmetryAxisForProject,
  symmetrySide,
} from '../../../lib/symmetryAuthoring';
import {
  createBlankTree,
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
    symmetryAuthoringPairs: [],

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
          foldArtifacts: null,
          foldArtifactError: null,
        });
        get().commitHistoryCheckpoint(checkpoint, 'Add node');
        void get().autosaveProject();
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
      }
    },

    addNodeWithSymmetry: async (loc, connectTo) => {
      const project = get().project;
      if (!project.hasSymmetry) {
        await get().addNodeAt(loc, connectTo);
        return;
      }

      set({ error: null });
      const checkpoint = await get().beginHistoryCheckpoint();
      try {
        const { api, treeHandle } = await requireActiveTree();
        const axis = symmetryAxisForProject(project);
        const snapped = snapPointToSymmetryAxis(loc, axis);
        const parent = connectTo === undefined ? null : project.nodes.find((node) => node.id === connectTo);
        const parentSide = parent ? symmetrySide(parent.loc, axis) : 0;
        const parentPair = parent
          ? findMirrorNodeId(project, get().symmetryAuthoringPairs, parent.id)
          : null;
        const shouldMirror = Boolean(parent && !snapped.snapped && (parentSide === 0 || parentPair));
        let snapshot: TreeSnapshot | null = null;
        let selection = get().selection;
        let authoringPairs = get().symmetryAuthoringPairs;
        if (parent && parentPair) {
          authoringPairs = addSymmetryAuthoringPair(authoringPairs, parent.id, parentPair);
        }

        const firstReport = await api.applyEdit(treeHandle, {
          type: 'add_node',
          loc: snapped.point,
          connect_to: connectTo,
          edge_length: connectTo === undefined ? undefined : 1,
        });
        snapshot = firstReport.snapshot;
        selection = nextSelectionForEdit(
          { type: 'add_node', loc: snapped.point, connect_to: connectTo },
          snapshot,
          firstReport.created_node,
          firstReport.created_edge
        );

        if (shouldMirror && firstReport.created_node) {
          const mirroredLoc = reflectPointAcrossSymmetryAxis(snapped.point, axis);
          const mirroredParent = parentSide === 0 ? connectTo : parentPair ?? undefined;
          const secondReport = await api.applyEdit(treeHandle, {
            type: 'add_node',
            loc: mirroredLoc,
            connect_to: mirroredParent,
            edge_length: mirroredParent === undefined ? undefined : 1,
          });
          snapshot = secondReport.snapshot;

          if (secondReport.created_node) {
            authoringPairs = addSymmetryAuthoringPair(
              authoringPairs,
              firstReport.created_node,
              secondReport.created_node
            );
            const conditionReport = await api.applyEdit(treeHandle, {
              type: 'add_condition',
              kind: {
                type: 'nodes_paired',
                node1: firstReport.created_node,
                node2: secondReport.created_node,
              },
            });
            snapshot = conditionReport.snapshot;
            selection = {
              kind: 'multi',
              nodes: [firstReport.created_node, secondReport.created_node].sort((a, b) => a - b),
              edges: [],
              paths: [],
              creases: [],
              facets: [],
              conditions: [],
            };
          }
        }

        if (!snapshot) return;
        const addedPair = selection.kind === 'multi' && selection.nodes.length === 2;
        const nextProject = projectFromSnapshot(snapshot, get().project.title);
        set({
          project: nextProject,
          symmetryAuthoringPairs: filterSymmetryAuthoringPairs(nextProject, authoringPairs),
          selection,
          status: statusAfterEdit(snapshot),
          dirty: true,
          error: null,
          lastOptimization: null,
          foldArtifacts: null,
          foldArtifactError: null,
          projectMessage: addedPair ? 'Added mirrored branch' : snapped.snapped ? 'Added axial node' : null,
        });
        get().commitHistoryCheckpoint(checkpoint, addedPair ? 'Add mirrored branch' : 'Add node');
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
          foldArtifacts: null,
          foldArtifactError: null,
        });
        get().commitHistoryCheckpoint(checkpoint, 'Move node');
        void get().autosaveProject();
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
      }
    },

    moveNodeWithSymmetry: async (id, loc) => {
      const project = get().project;
      const pairedNode = project.hasSymmetry
        ? findMirrorNodeId(project, get().symmetryAuthoringPairs, id)
        : null;
      if (!pairedNode) {
        await get().moveNode(id, loc);
        return;
      }

      set({ error: null });
      const checkpoint = await get().beginHistoryCheckpoint();
      try {
        const { api, treeHandle } = await requireActiveTree();
        const axis = symmetryAxisForProject(project);
        const edit: TreeEdit = { type: 'move_node', id, loc };
        const primaryReport = await api.applyEdit(treeHandle, edit);
        const pairedReport = await api.applyEdit(treeHandle, {
          type: 'move_node',
          id: pairedNode,
          loc: reflectPointAcrossSymmetryAxis(loc, axis),
        });
        set({
          project: projectFromSnapshot(pairedReport.snapshot, get().project.title),
          selection: nextSelectionForEdit(edit, primaryReport.snapshot),
          status: statusAfterEdit(pairedReport.snapshot),
          dirty: true,
          error: null,
          lastOptimization: null,
          foldArtifacts: null,
          foldArtifactError: null,
        });
        get().commitHistoryCheckpoint(checkpoint, 'Move mirrored nodes');
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
          foldArtifacts: null,
          foldArtifactError: null,
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
          foldArtifacts: null,
          foldArtifactError: null,
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
        const mirrorEdge = findMirrorEdgeId(get().project, get().symmetryAuthoringPairs, id);
        const mirrorUpdate = {
          length: update.length,
          stiffness: update.stiffness,
        };
        const shouldUpdateMirror =
          mirrorEdge !== null &&
          (mirrorUpdate.length !== undefined || mirrorUpdate.stiffness !== undefined);
        const finalReport = shouldUpdateMirror
          ? await api.applyEdit(treeHandle, {
              type: 'update_edge',
              id: mirrorEdge,
              ...mirrorUpdate,
            })
          : report;
        set({
          project: projectFromSnapshot(finalReport.snapshot, get().project.title),
          selection: nextSelectionForEdit(edit, report.snapshot),
          status: statusAfterEdit(finalReport.snapshot),
          dirty: true,
          error: null,
          lastOptimization: null,
          foldArtifacts: null,
          foldArtifactError: null,
        });
        get().commitHistoryCheckpoint(checkpoint, shouldUpdateMirror ? 'Edit mirrored edges' : 'Edit edge');
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
        if (selectionCoversAllNodes(selection, get().project)) {
          const snapshot = await createBlankTree(api);
          set({
            project: projectFromSnapshot(snapshot, get().project.title),
            selection: { kind: 'tree' },
            status: statusAfterEdit(snapshot),
            dirty: true,
            error: null,
            lastOptimization: null,
            foldArtifacts: null,
            foldArtifactError: null,
            projectMessage: 'Cleared design',
          });
          get().commitHistoryCheckpoint(checkpoint, 'Clear design');
          void get().autosaveProject();
          return;
        }

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
          foldArtifacts: null,
          foldArtifactError: null,
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
