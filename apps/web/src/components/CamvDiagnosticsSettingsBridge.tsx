import { useEffect, useRef } from 'react';
import { useSettingsStore } from '../store/settingsStore';
import { useWorkspaceStore } from '../store/workspaceStore';

export function CamvDiagnosticsSettingsBridge() {
  const camvAngleTolerance = useSettingsStore((state) => state.camvAngleTolerance);
  const hasOristudioCpDocument = useWorkspaceStore((state) => state.oristudioCpDocument !== null);
  const refreshOristudioCpCamvDiagnostics = useWorkspaceStore(
    (state) => state.refreshOristudioCpCamvDiagnostics
  );
  const previousCamvAngleTolerance = useRef(camvAngleTolerance);

  useEffect(() => {
    if (previousCamvAngleTolerance.current === camvAngleTolerance) return;
    previousCamvAngleTolerance.current = camvAngleTolerance;
    if (!hasOristudioCpDocument) return;

    void refreshOristudioCpCamvDiagnostics();
  }, [camvAngleTolerance, hasOristudioCpDocument, refreshOristudioCpCamvDiagnostics]);

  return null;
}
