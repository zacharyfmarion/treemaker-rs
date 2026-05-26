import { create } from 'zustand';
import { devtools } from 'zustand/middleware';

export type SettingsTab = 'appearance' | 'diagnostics' | 'workspace';

export const DEFAULT_CAMV_ANGLE_TOLERANCE = 0.000001;
export const MIN_CAMV_ANGLE_TOLERANCE = 0.000001;
export const MAX_CAMV_ANGLE_TOLERANCE = 10;
export const CAMV_ANGLE_TOLERANCE_STORAGE_KEY = 'treemaker.camvAngleTolerance.v1';

export function normalizeCamvAngleTolerance(value: unknown): number {
  const parsed =
    typeof value === 'number'
      ? value
      : typeof value === 'string'
        ? Number.parseFloat(value)
        : Number.NaN;
  if (!Number.isFinite(parsed)) return DEFAULT_CAMV_ANGLE_TOLERANCE;
  return Math.min(MAX_CAMV_ANGLE_TOLERANCE, Math.max(MIN_CAMV_ANGLE_TOLERANCE, parsed));
}

function loadCamvAngleTolerance(): number {
  if (typeof localStorage === 'undefined') return DEFAULT_CAMV_ANGLE_TOLERANCE;
  try {
    const saved = localStorage.getItem(CAMV_ANGLE_TOLERANCE_STORAGE_KEY);
    return saved === null ? DEFAULT_CAMV_ANGLE_TOLERANCE : normalizeCamvAngleTolerance(saved);
  } catch {
    return DEFAULT_CAMV_ANGLE_TOLERANCE;
  }
}

function saveCamvAngleTolerance(value: number): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(CAMV_ANGLE_TOLERANCE_STORAGE_KEY, String(value));
  } catch {
    // Ignore storage failures; the in-memory setting still applies for this session.
  }
}

function clearCamvAngleTolerance(): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.removeItem(CAMV_ANGLE_TOLERANCE_STORAGE_KEY);
  } catch {
    // Ignore storage failures; the in-memory setting still applies for this session.
  }
}

interface SettingsState {
  isSettingsOpen: boolean;
  settingsInitialTab: SettingsTab | null;
  camvAngleTolerance: number;
  openSettings: (tab?: SettingsTab) => void;
  closeSettings: () => void;
  setCamvAngleTolerance: (value: number) => void;
  resetCamvAngleTolerance: () => void;
}

export const useSettingsStore = create<SettingsState>()(
  devtools(
    (set) => ({
      isSettingsOpen: false,
      settingsInitialTab: null,
      camvAngleTolerance: loadCamvAngleTolerance(),
      openSettings: (tab) => set({ isSettingsOpen: true, settingsInitialTab: tab ?? null }),
      closeSettings: () => set({ isSettingsOpen: false, settingsInitialTab: null }),
      setCamvAngleTolerance: (value) => {
        const normalized = normalizeCamvAngleTolerance(value);
        saveCamvAngleTolerance(normalized);
        set({ camvAngleTolerance: normalized });
      },
      resetCamvAngleTolerance: () => {
        clearCamvAngleTolerance();
        set({ camvAngleTolerance: DEFAULT_CAMV_ANGLE_TOLERANCE });
      },
    }),
    { name: 'SettingsStore' }
  )
);
