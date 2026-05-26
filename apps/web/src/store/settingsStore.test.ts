import { beforeEach, describe, expect, it } from 'vitest';
import {
  CAMV_ANGLE_TOLERANCE_STORAGE_KEY,
  DEFAULT_CAMV_ANGLE_TOLERANCE,
  MIN_CAMV_ANGLE_TOLERANCE,
  useSettingsStore,
} from './settingsStore';

const initialSettingsState = useSettingsStore.getInitialState();

beforeEach(() => {
  localStorage.clear();
  useSettingsStore.setState(initialSettingsState, true);
});

describe('settingsStore', () => {
  it('opens and closes the settings modal', () => {
    useSettingsStore.getState().openSettings();

    expect(useSettingsStore.getState().isSettingsOpen).toBe(true);
    expect(useSettingsStore.getState().settingsInitialTab).toBeNull();

    useSettingsStore.getState().closeSettings();

    expect(useSettingsStore.getState().isSettingsOpen).toBe(false);
    expect(useSettingsStore.getState().settingsInitialTab).toBeNull();
  });

  it('tracks the requested initial tab', () => {
    useSettingsStore.getState().openSettings('workspace');

    expect(useSettingsStore.getState().isSettingsOpen).toBe(true);
    expect(useSettingsStore.getState().settingsInitialTab).toBe('workspace');
  });

  it('normalizes and persists the CAMV angle tolerance', () => {
    useSettingsStore.getState().setCamvAngleTolerance(0.25);

    expect(useSettingsStore.getState().camvAngleTolerance).toBe(0.25);
    expect(localStorage.getItem(CAMV_ANGLE_TOLERANCE_STORAGE_KEY)).toBe('0.25');

    useSettingsStore.getState().setCamvAngleTolerance(-1);
    expect(useSettingsStore.getState().camvAngleTolerance).toBe(MIN_CAMV_ANGLE_TOLERANCE);

    useSettingsStore.getState().resetCamvAngleTolerance();
    expect(useSettingsStore.getState().camvAngleTolerance).toBe(DEFAULT_CAMV_ANGLE_TOLERANCE);
    expect(localStorage.getItem(CAMV_ANGLE_TOLERANCE_STORAGE_KEY)).toBeNull();
  });
});
