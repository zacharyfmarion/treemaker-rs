import { createEmptyProject } from '../../../lib/sampleProject';
import { useLayoutStore } from '../../layoutStore';
import {
  createBlankTree,
  createStarterTree,
  engineError,
  getEngine,
  initializeBlankTree,
  projectStateFromSnapshot,
} from '../engineRuntime';
import type { ProjectSlice, WorkspaceSliceCreator } from '../types';

export const createProjectSlice: WorkspaceSliceCreator<ProjectSlice> = (set) => ({
  project: createEmptyProject(),
  status: 'loading_engine',
  dirty: false,
  engineReady: false,
  error: null,
  lastOptimization: null,

  initEngine: async () => {
    set({ status: 'loading_engine', error: null });
    try {
      const api = await getEngine();
      const snapshot = await initializeBlankTree(api);
      set({
        ...projectStateFromSnapshot(snapshot),
        selection: { kind: 'tree' },
        dirty: false,
        lastOptimization: null,
      });
    } catch (error) {
      set({ status: 'error', error: engineError(error), engineReady: false });
    }
  },

  createNewProject: async () => {
    set({ status: 'loading_engine', error: null });
    try {
      const api = await getEngine();
      const snapshot = await createBlankTree(api);
      set({
        ...projectStateFromSnapshot(snapshot),
        selection: { kind: 'tree' },
        toolMode: 'select',
        creaseColorMode: 'mvf',
        dirty: false,
        lastOptimization: null,
      });
      useLayoutStore.getState().activatePanel('design');
    } catch (error) {
      set({ status: 'error', error: engineError(error) });
    }
  },

  loadStarterProject: async () => {
    set({ status: 'loading_engine', error: null });
    try {
      const api = await getEngine();
      const snapshot = await createStarterTree(api);
      set({
        ...projectStateFromSnapshot(snapshot),
        selection: { kind: 'tree' },
        toolMode: 'select',
        creaseColorMode: 'mvf',
        dirty: false,
        lastOptimization: null,
      });
      useLayoutStore.getState().activatePanel('design');
    } catch (error) {
      set({ status: 'error', error: engineError(error) });
    }
  },
});
