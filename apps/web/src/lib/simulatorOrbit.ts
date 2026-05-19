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

export function nextSimulatorOrbitView(
  view: SimulatorOrbitView,
  drag: SimulatorOrbitDrag,
  point: SimulatorOrbitPoint
): SimulatorOrbitView {
  return {
    ...view,
    yaw: normalizeAngle(drag.yaw - (point.x - drag.x) * SIMULATOR_ORBIT_SENSITIVITY),
    pitch: normalizeAngle(drag.pitch + (point.y - drag.y) * SIMULATOR_ORBIT_SENSITIVITY),
  };
}

export function normalizeAngle(value: number): number {
  const fullTurn = Math.PI * 2;
  return ((((value + Math.PI) % fullTurn) + fullTurn) % fullTurn) - Math.PI;
}
