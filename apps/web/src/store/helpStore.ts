import { create } from 'zustand';
import { devtools } from 'zustand/middleware';

export type HelpModalKind = 'guide' | 'about';

interface HelpState {
  activeModal: HelpModalKind | null;
  openGuide: () => void;
  openAbout: () => void;
  closeHelp: () => void;
}

export const useHelpStore = create<HelpState>()(
  devtools(
    (set) => ({
      activeModal: null,
      openGuide: () => set({ activeModal: 'guide' }),
      openAbout: () => set({ activeModal: 'about' }),
      closeHelp: () => set({ activeModal: null }),
    }),
    { name: 'HelpStore' }
  )
);
