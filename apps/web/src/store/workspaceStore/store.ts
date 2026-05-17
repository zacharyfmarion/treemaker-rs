import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import { createCreasePatternSlice } from './slices/creasePatternSlice';
import { createEditingSlice } from './slices/editingSlice';
import { createProjectSlice } from './slices/projectSlice';
import type { WorkspaceState } from './types';

export const useWorkspaceStore = create<WorkspaceState>()(
  devtools(
    (...args) => ({
      ...createProjectSlice(...args),
      ...createEditingSlice(...args),
      ...createCreasePatternSlice(...args),
    }),
    { name: 'treemaker-workspace' }
  )
);

if (import.meta.env.DEV && typeof window !== 'undefined') {
  const debugWindow = window as Window & {
    __treemakerWorkspaceStore?: typeof useWorkspaceStore;
  };
  debugWindow.__treemakerWorkspaceStore = useWorkspaceStore;
}
