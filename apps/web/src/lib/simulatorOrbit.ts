export interface SimulatorOrbitView {
  yaw: number;
  pitch: number;
  zoom: number;
}

export interface SimulatorOrbitDrag {
  x: number;
  y: number;
  yaw: number;
  pitch: number;
}

export interface SimulatorOrbitPoint {
  x: number;
  y: number;
}

export const SIMULATOR_ORBIT_SENSITIVITY = 0.01;
export const SIMULATOR_MIN_PITCH = -1.35;
export const SIMULATOR_MAX_PITCH = 1.35;

export function nextSimulatorOrbitView(
  view: SimulatorOrbitView,
  drag: SimulatorOrbitDrag,
  point: SimulatorOrbitPoint
): SimulatorOrbitView {
  return {
    ...view,
    yaw: drag.yaw - (point.x - drag.x) * SIMULATOR_ORBIT_SENSITIVITY,
    pitch: clamp(
      drag.pitch + (point.y - drag.y) * SIMULATOR_ORBIT_SENSITIVITY,
      SIMULATOR_MIN_PITCH,
      SIMULATOR_MAX_PITCH
    ),
  };
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
