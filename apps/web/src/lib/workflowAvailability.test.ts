import { describe, expect, it } from 'vitest';
import type { AppStatus } from './sampleProject';
import { getCreasePatternWorkflowState } from './workflowAvailability';

function workflow(status: AppStatus, edgeCount: number, engineReady = true) {
  return getCreasePatternWorkflowState({ engineReady, status, edgeCount });
}

describe('crease-pattern workflow availability', () => {
  it('disables optimize and build when there are no tree edges', () => {
    const state = workflow('ready', 0);

    expect(state.canOptimizeScale).toBe(false);
    expect(state.optimizeScaleReason).toBe('Add at least one tree edge before optimizing');
    expect(state.canBuildCreasePattern).toBe(false);
    expect(state.buildCreasePatternReason).toBe(
      'Add tree edges, then optimize before building the crease pattern'
    );
  });

  it('enables optimization but blocks Build CP before optimization succeeds', () => {
    const state = workflow('needs_optimization', 2);

    expect(state.canOptimizeScale).toBe(true);
    expect(state.canBuildCreasePattern).toBe(false);
    expect(state.buildCreasePatternReason).toBe('Optimize Scale before building the crease pattern');
  });

  it('enables Build CP after optimization succeeds', () => {
    const state = workflow('optimized', 2);

    expect(state.canOptimizeScale).toBe(true);
    expect(state.canBuildCreasePattern).toBe(true);
    expect(state.buildCreasePatternReason).toBe('Build crease pattern');
  });

  it('allows rebuilding an existing crease pattern', () => {
    const state = workflow('crease_pattern_ready', 2);

    expect(state.canBuildCreasePattern).toBe(true);
    expect(state.buildCreasePatternReason).toBe('Rebuild crease pattern');
  });

  it('disables workflow actions while the engine is busy or unavailable', () => {
    for (const status of ['loading_engine', 'optimizing', 'building_crease_pattern'] as const) {
      const state = workflow(status, 2, status !== 'loading_engine');

      expect(state.isBusy).toBe(true);
      expect(state.canOptimizeScale).toBe(false);
      expect(state.canBuildCreasePattern).toBe(false);
    }

    const errorState = workflow('error', 2);
    expect(errorState.canOptimizeScale).toBe(false);
    expect(errorState.canBuildCreasePattern).toBe(false);
  });
});
