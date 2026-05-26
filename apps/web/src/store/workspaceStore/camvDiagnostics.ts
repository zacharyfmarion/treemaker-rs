import type { OristudioCpCommandPayload } from '../../engine/oristudioCpTypes';
import type { OristudioCpOperationId } from '../../lib/oristudioCpCommands';
import { useSettingsStore } from '../settingsStore';

export const CAMV_ANGLE_TOLERANCE_OPERATIONS = new Set<OristudioCpOperationId>([
  'Check4',
  'CheckCamv',
]);

export function withCamvAngleTolerancePayload(
  payload: OristudioCpCommandPayload = {}
): OristudioCpCommandPayload {
  return {
    ...payload,
    camv_angle_tolerance:
      payload.camv_angle_tolerance ?? useSettingsStore.getState().camvAngleTolerance,
  };
}
