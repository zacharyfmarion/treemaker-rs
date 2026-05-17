import { create } from 'zustand';
import {
  createSampleProject,
  type AppStatus,
  type CreaseColorMode,
  type Selection,
  type ToolMode,
  type TreeProject,
} from '../lib/sampleProject';

interface WorkspaceState {
  project: TreeProject;
  selection: Selection;
  toolMode: ToolMode;
  creaseColorMode: CreaseColorMode;
  status: AppStatus;
  dirty: boolean;
  select: (selection: Selection) => void;
  setToolMode: (toolMode: ToolMode) => void;
  setCreaseColorMode: (mode: CreaseColorMode) => void;
  createNewProject: () => void;
}

export const useWorkspaceStore = create<WorkspaceState>((set) => ({
  project: createSampleProject(),
  selection: { kind: 'tree' },
  toolMode: 'select',
  creaseColorMode: 'mvf',
  status: 'ready',
  dirty: false,
  select: (selection) => set({ selection }),
  setToolMode: (toolMode) => set({ toolMode }),
  setCreaseColorMode: (creaseColorMode) => set({ creaseColorMode }),
  createNewProject: () =>
    set({
      project: createSampleProject(),
      selection: { kind: 'tree' },
      toolMode: 'select',
      creaseColorMode: 'mvf',
      status: 'ready',
      dirty: false,
    }),
}));
