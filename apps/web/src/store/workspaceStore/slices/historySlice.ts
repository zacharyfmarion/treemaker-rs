import {
  engineError,
  ensureTreeHandle,
  getEngine,
  loadTreeFromText,
  projectStateFromSnapshot,
  statusFromSnapshot,
} from '../engineRuntime';
import type { HistoryEntry, HistorySlice, WorkspaceSliceCreator } from '../types';

const MAX_HISTORY = 100;

function historyEntry(text: string, label = 'Edit'): HistoryEntry {
  return {
    text,
    label,
    timestamp: new Date().toISOString(),
  };
}

export const createHistorySlice: WorkspaceSliceCreator<HistorySlice> = (set, get) => ({
  historyPast: [],
  historyFuture: [],
  historyBusy: false,

  beginHistoryCheckpoint: async () => {
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

  clearHistory: () => set({ historyPast: [], historyFuture: [] }),

  undo: async () => {
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
