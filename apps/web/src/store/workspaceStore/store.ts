import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import { createCreasePatternSlice } from './slices/creasePatternSlice';
import { createClipboardSlice } from './slices/clipboardSlice';
import { createConditionSlice } from './slices/conditionSlice';
import { createEditingSlice } from './slices/editingSlice';
import { createHistorySlice } from './slices/historySlice';
import { createProjectSlice } from './slices/projectSlice';
import type { WorkspaceState } from './types';

export const useWorkspaceStore = create<WorkspaceState>()(
  devtools(
    (...args) => ({
      ...createProjectSlice(...args),
      ...createHistorySlice(...args),
      ...createEditingSlice(...args),
      ...createClipboardSlice(...args),
      ...createConditionSlice(...args),
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
