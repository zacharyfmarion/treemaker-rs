import type { AppStatus } from './sampleProject';

export interface CreasePatternWorkflowInput {
  engineReady: boolean;
  status: AppStatus;
  edgeCount: number;
}

export interface CreasePatternWorkflowState {
  canOptimizeScale: boolean;
  canBuildCreasePattern: boolean;
  optimizeScaleReason: string;
  buildCreasePatternReason: string;
  isBusy: boolean;
  hasTreeEdges: boolean;
}

export function getCreasePatternWorkflowState({
  engineReady,
  status,
  edgeCount,
}: CreasePatternWorkflowInput): CreasePatternWorkflowState {
  const isBusy =
    status === 'loading_engine' ||
    status === 'optimizing' ||
    status === 'building_crease_pattern';
  const hasTreeEdges = edgeCount > 0;
  const canOptimizeScale = engineReady && hasTreeEdges && !isBusy && status !== 'error';
  const canBuildCreasePattern =
    engineReady && !isBusy && (status === 'optimized' || status === 'crease_pattern_ready');

  return {
    canOptimizeScale,
    canBuildCreasePattern,
    optimizeScaleReason: canOptimizeScale
      ? 'Optimize Scale'
      : disabledOptimizeScaleReason({ engineReady, status, hasTreeEdges, isBusy }),
    buildCreasePatternReason: canBuildCreasePattern
      ? status === 'crease_pattern_ready'
        ? 'Rebuild crease pattern'
        : 'Build crease pattern'
      : disabledBuildCreasePatternReason({ engineReady, status, hasTreeEdges, isBusy }),
    isBusy,
    hasTreeEdges,
  };
}

function disabledOptimizeScaleReason({
  engineReady,
  status,
  hasTreeEdges,
  isBusy,
}: {
  engineReady: boolean;
  status: AppStatus;
  hasTreeEdges: boolean;
  isBusy: boolean;
}): string {
  if (!engineReady || status === 'loading_engine') return 'TreeMaker engine is still loading';
  if (status === 'optimizing') return 'Optimization is running';
  if (status === 'building_crease_pattern') return 'Crease pattern build is running';
  if (isBusy) return 'TreeMaker is busy';
  if (!hasTreeEdges) return 'Add at least one tree edge before optimizing';
  if (status === 'error') return 'Resolve the current TreeMaker error before optimizing';
  return 'Optimization is unavailable';
}

function disabledBuildCreasePatternReason({
  engineReady,
  status,
  hasTreeEdges,
  isBusy,
}: {
  engineReady: boolean;
  status: AppStatus;
  hasTreeEdges: boolean;
  isBusy: boolean;
}): string {
  if (!engineReady || status === 'loading_engine') return 'TreeMaker engine is still loading';
  if (status === 'optimizing') return 'Optimization is running';
  if (status === 'building_crease_pattern') return 'Crease pattern build is running';
  if (isBusy) return 'TreeMaker is busy';
  if (status === 'error') return 'Resolve the current TreeMaker error before building the crease pattern';
  if (!hasTreeEdges) return 'Add tree edges, then optimize before building the crease pattern';
  return 'Optimize Scale before building the crease pattern';
}
