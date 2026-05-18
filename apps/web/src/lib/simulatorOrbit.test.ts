import { describe, expect, it } from 'vitest';
import {
  SIMULATOR_MAX_PITCH,
  SIMULATOR_MIN_PITCH,
  nextSimulatorOrbitView,
  type SimulatorOrbitDrag,
  type SimulatorOrbitView,
} from './simulatorOrbit';

describe('simulator orbit controls', () => {
  const view: SimulatorOrbitView = { yaw: 1, pitch: 0.2, zoom: 1.5 };
  const drag: SimulatorOrbitDrag = { x: 100, y: 100, yaw: view.yaw, pitch: view.pitch };

  it('maps horizontal dragging to conventional orbit direction', () => {
    expect(nextSimulatorOrbitView(view, drag, { x: 140, y: 100 }).yaw).toBeLessThan(view.yaw);
    expect(nextSimulatorOrbitView(view, drag, { x: 60, y: 100 }).yaw).toBeGreaterThan(view.yaw);
  });

  it('maps vertical dragging to conventional orbit direction', () => {
    expect(nextSimulatorOrbitView(view, drag, { x: 100, y: 140 }).pitch).toBeGreaterThan(view.pitch);
    expect(nextSimulatorOrbitView(view, drag, { x: 100, y: 60 }).pitch).toBeLessThan(view.pitch);
  });

  it('preserves zoom and clamps pitch', () => {
    expect(nextSimulatorOrbitView(view, drag, { x: 100, y: 100 }).zoom).toBe(view.zoom);
    expect(nextSimulatorOrbitView(view, drag, { x: 100, y: -1000 }).pitch).toBe(SIMULATOR_MIN_PITCH);
    expect(nextSimulatorOrbitView(view, drag, { x: 100, y: 1000 }).pitch).toBe(SIMULATOR_MAX_PITCH);
  });
});
