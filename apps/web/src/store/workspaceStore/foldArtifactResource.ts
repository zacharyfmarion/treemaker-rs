import type { FoldArtifacts } from '../../engine/types';
import type { SequenceSimulationFocus } from './types';

export type FoldArtifactStatus = 'idle' | 'stale' | 'loading' | 'ready' | 'error';

export interface FoldArtifactResourceState {
  foldArtifacts: FoldArtifacts | null;
  foldArtifactError: string | null;
  foldArtifactStatus: FoldArtifactStatus;
  foldArtifactRevision: number;
  foldArtifactResolvedRevision: number | null;
  foldArtifactRequestId: number;
}

export interface FoldArtifactDependentState {
  sequenceTarget: null;
  sequencePlan: null;
  sequenceSimulationFocus: SequenceSimulationFocus;
  sequencePlanning: false;
  sequenceError: null;
}

const wholeSimulationFocus: SequenceSimulationFocus = { kind: 'whole' };

export function emptyFoldArtifactResourceState(): FoldArtifactResourceState {
  return {
    foldArtifacts: null,
    foldArtifactError: null,
    foldArtifactStatus: 'idle',
    foldArtifactRevision: 0,
    foldArtifactResolvedRevision: null,
    foldArtifactRequestId: 0,
  };
}

export function staleFoldArtifactResourceState(
  currentRevision: number
): FoldArtifactResourceState & FoldArtifactDependentState {
  return {
    foldArtifacts: null,
    foldArtifactError: null,
    foldArtifactStatus: 'stale',
    foldArtifactRevision: currentRevision + 1,
    foldArtifactResolvedRevision: null,
    foldArtifactRequestId: 0,
    sequenceTarget: null,
    sequencePlan: null,
    sequenceSimulationFocus: wholeSimulationFocus,
    sequencePlanning: false,
    sequenceError: null,
  };
}

export function readyFoldArtifactResourceState(
  foldArtifacts: FoldArtifacts,
  revision: number
): Omit<FoldArtifactResourceState, 'foldArtifactRequestId'> {
  return {
    foldArtifacts,
    foldArtifactError: null,
    foldArtifactStatus: 'ready',
    foldArtifactRevision: revision,
    foldArtifactResolvedRevision: revision,
  };
}
