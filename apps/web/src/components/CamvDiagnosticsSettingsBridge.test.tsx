import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest';
import { useSettingsStore } from '../store/settingsStore';

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

const workspaceMock = vi.hoisted(() => ({
  state: {
    oristudioCpDocument: null as unknown,
    refreshOristudioCpCamvDiagnostics: vi.fn(async () => true),
  },
}));

vi.mock('../store/workspaceStore', () => ({
  useWorkspaceStore: <T,>(
    selector: (state: typeof workspaceMock.state) => T
  ): T => selector(workspaceMock.state),
}));

import { CamvDiagnosticsSettingsBridge } from './CamvDiagnosticsSettingsBridge';

const initialSettingsState = useSettingsStore.getInitialState();
let root: Root | null = null;
let container: HTMLDivElement | null = null;

function renderBridge() {
  container = document.createElement('div');
  document.body.append(container);
  root = createRoot(container);
  act(() => {
    root?.render(<CamvDiagnosticsSettingsBridge />);
  });
}

beforeEach(() => {
  localStorage.clear();
  useSettingsStore.setState(initialSettingsState, true);
  workspaceMock.state.oristudioCpDocument = null;
  workspaceMock.state.refreshOristudioCpCamvDiagnostics.mockClear();
});

afterEach(() => {
  if (root) {
    act(() => {
      root?.unmount();
    });
  }
  container?.remove();
  root = null;
  container = null;
});

describe('CamvDiagnosticsSettingsBridge', () => {
  it('refreshes editable CP CAMV diagnostics when the tolerance changes', async () => {
    workspaceMock.state.oristudioCpDocument = { handle: 7 };
    renderBridge();

    await act(async () => {
      useSettingsStore.getState().setCamvAngleTolerance(0.25);
      await Promise.resolve();
    });

    expect(workspaceMock.state.refreshOristudioCpCamvDiagnostics).toHaveBeenCalledOnce();
  });

  it('does not refresh when no editable CP document is loaded', async () => {
    renderBridge();

    await act(async () => {
      useSettingsStore.getState().setCamvAngleTolerance(0.25);
      await Promise.resolve();
    });

    expect(workspaceMock.state.refreshOristudioCpCamvDiagnostics).not.toHaveBeenCalled();
  });
});
