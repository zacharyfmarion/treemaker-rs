import { describe, expect, it } from 'vitest';
import { simulatorRunConfig } from './simulatorRunConfig';

describe('simulatorRunConfig', () => {
  it('leaves whole-model simulator timing on the existing standard preset', () => {
    const whole = simulatorRunConfig('whole', 'accurate');

    expect(whole.initialSettleSteps).toBe(300);
    expect(whole.foldStepPercent).toBe(5);
    expect(whole.solverOptions).toEqual({});
  });

  it('uses more work and a smaller adaptive timestep for accurate step simulation', () => {
    const fast = simulatorRunConfig('step', 'fast');
    const accurate = simulatorRunConfig('step', 'accurate');

    expect(accurate.initialSettleSteps).toBeGreaterThan(fast.initialSettleSteps);
    expect(accurate.foldChangeImmediateSteps).toBeGreaterThan(fast.foldChangeImmediateSteps);
    expect(accurate.foldStepPercent).toBeLessThan(fast.foldStepPercent);
    expect(accurate.foldPlayPercentPerSecond).toBeLessThan(fast.foldPlayPercentPerSecond);
    expect(accurate.solverOptions.timeStepScale).toBeLessThan(1);
  });
});
