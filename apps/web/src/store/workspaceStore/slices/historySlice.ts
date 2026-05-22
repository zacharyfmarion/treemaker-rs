import {
  engineError,
  ensureTreeHandle,
  getEngine,
  loadTreeFromText,
  projectStateFromSnapshot,
  statusFromSnapshot,
} from '../engineRuntime';
import { oristudioCpError, restoreOristudioCpDocument } from '../oristudioCpRuntime';
import type {
  HistoryEntry,
  HistorySlice,
  OristudioCpHistoryEntry,
  WorkspaceSliceCreator,
} from '../types';
import type {
  OristudioCpDocumentSnapshot,
  OristudioCpDocumentState,
} from '../../../engine/oristudioCpTypes';
import type { OristudioCpSelection } from '../../../lib/creasePatternViewport';

const MAX_HISTORY = 100;

function historyEntry(text: string, label = 'Edit'): HistoryEntry {
  return {
    text,
    label,
    timestamp: new Date().toISOString(),
  };
}

function cpHistoryEntry(
  document: OristudioCpDocumentSnapshot,
  selection: OristudioCpSelection,
  label = 'Edit'
): OristudioCpHistoryEntry {
  return {
    document,
    selection,
    label,
    timestamp: new Date().toISOString(),
  };
}

function setRestoredCreasePatternState(
  restored: OristudioCpDocumentState,
  selection: OristudioCpSelection
) {
  return {
    oristudioCpDocument: restored,
    oristudioCpOperationDescriptors: restored.operationDescriptors,
    oristudioCpSelection: selection,
    oristudioCpError: null,
    error: null,
    dirty: true,
    status: 'crease_pattern_ready' as const,
  };
}

export const createHistorySlice: WorkspaceSliceCreator<HistorySlice> = (set, get) => ({
  historyPast: [],
  historyFuture: [],
  historyBusy: false,

  beginHistoryCheckpoint: async () => {
    if (get().documentMode !== 'tree') return null;
    try {
      const { api, treeHandle } = await ensureTreeHandle();
      return api.saveTmd5(treeHandle);
    } catch {
      return null;
    }
  },

  commitHistoryCheckpoint: (beforeText, label = 'Edit') => {
    if (!beforeText || get().historyBusy) return;
    const past = get().historyPast;
    if (past.at(-1)?.text === beforeText) {
      set({ historyFuture: [] });
      return;
    }
    set({
      historyPast: [...past, historyEntry(beforeText, label)].slice(-MAX_HISTORY),
      historyFuture: [],
    });
  },

  clearHistory: () =>
    set({
      historyPast: [],
      historyFuture: [],
      oristudioCpHistoryPast: [],
      oristudioCpHistoryFuture: [],
    }),

  undo: async () => {
    if (get().documentMode === 'crease-pattern') {
      const past = get().oristudioCpHistoryPast;
      const previous = past.at(-1);
      const current = get().oristudioCpDocument;
      if (!previous || !current || get().historyBusy) return;
      const currentSelection = get().oristudioCpSelection;
      set({ historyBusy: true, error: null, oristudioCpError: null });
      try {
        const restored = await restoreOristudioCpDocument(previous.document, current.source, null);
        set({
          ...setRestoredCreasePatternState(restored, previous.selection),
          oristudioCpHistoryPast: past.slice(0, -1),
          oristudioCpHistoryFuture: [
            cpHistoryEntry(current.document, currentSelection, previous.label),
            ...get().oristudioCpHistoryFuture,
          ].slice(0, MAX_HISTORY),
          historyBusy: false,
          projectMessage: `Undid ${previous.label}`,
        });
      } catch (error) {
        const normalized = oristudioCpError(error);
        set({
          status: 'error',
          error: normalized,
          oristudioCpError: normalized.message,
          historyBusy: false,
        });
      }
      return;
    }

    if (get().documentMode !== 'tree') return;
    const past = get().historyPast;
    const previous = past.at(-1);
    if (!previous || get().historyBusy) return;
    set({ historyBusy: true, error: null });
    try {
      const { api, treeHandle } = await ensureTreeHandle();
      const current = await api.saveTmd5(treeHandle);
      const engine = await getEngine();
      const snapshot = await loadTreeFromText(engine, previous.text);
      set({
        ...projectStateFromSnapshot(snapshot, get().project.title),
        historyPast: past.slice(0, -1),
        historyFuture: [historyEntry(current, previous.label), ...get().historyFuture].slice(
          0,
          MAX_HISTORY
        ),
        historyBusy: false,
        selection: { kind: 'tree' },
        symmetryAuthoringPairs: [],
        status: statusFromSnapshot(snapshot),
        dirty: true,
        projectMessage: `Undid ${previous.label}`,
        lastOptimization: null,
      });
      void get().autosaveProject();
    } catch (error) {
      set({ status: 'error', error: engineError(error), historyBusy: false });
    }
  },

  redo: async () => {
    if (get().documentMode === 'crease-pattern') {
      const future = get().oristudioCpHistoryFuture;
      const next = future[0];
      const current = get().oristudioCpDocument;
      if (!next || !current || get().historyBusy) return;
      const currentSelection = get().oristudioCpSelection;
      set({ historyBusy: true, error: null, oristudioCpError: null });
      try {
        const restored = await restoreOristudioCpDocument(next.document, current.source, null);
        set({
          ...setRestoredCreasePatternState(restored, next.selection),
          oristudioCpHistoryPast: [
            ...get().oristudioCpHistoryPast,
            cpHistoryEntry(current.document, currentSelection, next.label),
          ].slice(-MAX_HISTORY),
          oristudioCpHistoryFuture: future.slice(1),
          historyBusy: false,
          projectMessage: `Redid ${next.label}`,
        });
      } catch (error) {
        const normalized = oristudioCpError(error);
        set({
          status: 'error',
          error: normalized,
          oristudioCpError: normalized.message,
          historyBusy: false,
        });
      }
      return;
    }

    if (get().documentMode !== 'tree') return;
    const future = get().historyFuture;
    const next = future[0];
    if (!next || get().historyBusy) return;
    set({ historyBusy: true, error: null });
    try {
      const { api, treeHandle } = await ensureTreeHandle();
      const current = await api.saveTmd5(treeHandle);
      const engine = await getEngine();
      const snapshot = await loadTreeFromText(engine, next.text);
      set({
        ...projectStateFromSnapshot(snapshot, get().project.title),
        historyPast: [...get().historyPast, historyEntry(current, next.label)].slice(-MAX_HISTORY),
        historyFuture: future.slice(1),
        historyBusy: false,
        selection: { kind: 'tree' },
        symmetryAuthoringPairs: [],
        status: statusFromSnapshot(snapshot),
        dirty: true,
        projectMessage: `Redid ${next.label}`,
        lastOptimization: null,
      });
      void get().autosaveProject();
    } catch (error) {
      set({ status: 'error', error: engineError(error), historyBusy: false });
    }
  },
});
