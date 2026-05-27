import { create } from 'zustand';
import { devtools } from 'zustand/middleware';

export type SettingsTab = 'appearance' | 'shortcuts' | 'workspace';

interface SettingsState {
  isSettingsOpen: boolean;
  settingsInitialTab: SettingsTab | null;
  openSettings: (tab?: SettingsTab) => void;
  closeSettings: () => void;
}

export const useSettingsStore = create<SettingsState>()(
  devtools(
    (set) => ({
      isSettingsOpen: false,
      settingsInitialTab: null,
      openSettings: (tab) => set({ isSettingsOpen: true, settingsInitialTab: tab ?? null }),
      closeSettings: () => set({ isSettingsOpen: false, settingsInitialTab: null }),
    }),
    { name: 'SettingsStore' }
  )
);
