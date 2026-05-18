import { OrigamiModel } from './model.js';
import type { SimulationFrame, SimulatorOptions } from './types.js';

const DEFAULT_OPTIONS: Required<SimulatorOptions> = {
  foldPercent: 0,
  axialStiffness: 20,
  creaseStiffness: 0.7,
  panelStiffness: 0.7,
  damping: 0.45,
  timeStep: 1 / 60,
  stepsPerFrame: 8,
  autoRender: true,
};

export class DynamicSolver {
  readonly model: OrigamiModel;
  options: Required<SimulatorOptions>;
  private currentStep = 0;

  constructor(model: OrigamiModel, options: SimulatorOptions = {}) {
    this.model = model;
    this.options = { ...DEFAULT_OPTIONS, ...options };
  }

  setFoldPercent(percent: number): void {
    this.options.foldPercent = Math.max(-100, Math.min(100, percent));
  }

  setMaterial(options: Partial<SimulatorOptions>): void {
    this.options = { ...this.options, ...options };
  }

  reset(): void {
    this.currentStep = 0;
    this.model.reset();
  }

  step(numSteps = this.options.stepsPerFrame): SimulationFrame {
    for (let i = 0; i < numSteps; i += 1) {
      this.solveStep();
    }
    this.model.applyStrainColors(0.05);
    return this.readFrame();
  }

  readFrame(): SimulationFrame {
    return {
      positions: this.model.positions.slice(),
      colors: this.model.colors.slice(),
      indices: this.model.prepared.indices,
      diagnostics: this.model.diagnostics(),
      step: this.currentStep,
      foldPercent: this.options.foldPercent,
    };
  }

  private solveStep(): void {
    const target = this.model.computeTarget(this.options.foldPercent);
    const stiffness = this.options.creaseStiffness;
    const damping = Math.max(0, Math.min(1, this.options.damping));
    const dt = this.options.timeStep;

    for (let index = 0; index < this.model.positions.length; index += 1) {
      const displacement = target[index] - this.model.positions[index];
      const velocity = (this.model.velocities[index] + displacement * stiffness * dt) * (1 - damping * dt);
      this.model.velocities[index] = velocity;
      this.model.positions[index] += velocity;
    }
    this.currentStep += 1;
  }
}
