import type { OristudioCpCommandPayload } from '../../engine/oristudioCpTypes';
import type { OristudioCpOperationId } from '../../lib/oristudioCpCommands';
import { useSettingsStore } from '../settingsStore';

export const CAMV_ANGLE_TOLERANCE_OPERATIONS = new Set<OristudioCpOperationId>([
  'Check4',
  'CheckCamv',
]);

export function currentCamvAngleTolerance(): number {
  return useSettingsStore.getState().camvAngleTolerance;
}

export function withCamvAngleTolerancePayload(
  payload: OristudioCpCommandPayload = {},
  angleTolerance = currentCamvAngleTolerance()
): OristudioCpCommandPayload {
  return {
    ...payload,
    camv_angle_tolerance: payload.camv_angle_tolerance ?? angleTolerance,
  };
}
