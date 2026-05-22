import type { SimulatorOptions } from '@treemaker/origami-simulator';

export type SimulatorScope = 'whole' | 'step';
export type StepSimulationAccuracy = 'fast' | 'accurate';

export interface SimulatorRunConfig {
  initialSettleSteps: number;
  foldChangeImmediateSteps: number;
  foldChangeSettleBatch: number;
  foldChangeSettleFrames: number;
  foldPlayStepBatch: number;
  foldPlayPercentPerSecond: number;
  foldStepPercent: number;
  solverOptions: Partial<SimulatorOptions>;
}

const WHOLE_RUN_CONFIG: SimulatorRunConfig = {
  initialSettleSteps: 300,
  foldChangeImmediateSteps: 200,
  foldChangeSettleBatch: 200,
  foldChangeSettleFrames: 40,
  foldPlayStepBatch: 160,
  foldPlayPercentPerSecond: 28,
  foldStepPercent: 5,
  solverOptions: {},
};

const STEP_FAST_RUN_CONFIG: SimulatorRunConfig = {
  ...WHOLE_RUN_CONFIG,
};

const STEP_ACCURATE_RUN_CONFIG: SimulatorRunConfig = {
  initialSettleSteps: 1200,
  foldChangeImmediateSteps: 900,
  foldChangeSettleBatch: 900,
  foldChangeSettleFrames: 90,
  foldPlayStepBatch: 520,
  foldPlayPercentPerSecond: 12,
  foldStepPercent: 2,
  solverOptions: {
    timeStepScale: 0.35,
    stepsPerFrame: 240,
  },
};

export const STEP_SIMULATION_ACCURACY_OPTIONS: Array<{
  value: StepSimulationAccuracy;
  label: string;
  title: string;
}> = [
  { value: 'fast', label: 'Fast', title: 'Step preview with standard simulator work' },
  {
    value: 'accurate',
    label: 'Accurate',
    title: 'Step preview with smaller solver increments and more settling',
  },
];

export function simulatorRunConfig(
  scope: SimulatorScope,
  stepAccuracy: StepSimulationAccuracy
): SimulatorRunConfig {
  if (scope === 'whole') return WHOLE_RUN_CONFIG;
  return stepAccuracy === 'accurate' ? STEP_ACCURATE_RUN_CONFIG : STEP_FAST_RUN_CONFIG;
}
