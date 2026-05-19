import { Play, Sparkles } from 'lucide-react';
import { getCreasePatternWorkflowState } from '../../lib/workflowAvailability';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { Button } from '../ui/Button';

export function CreasePatternWorkflowButton() {
  const edgeCount = useWorkspaceStore((state) => state.project.edges.length);
  const status = useWorkspaceStore((state) => state.status);
  const engineReady = useWorkspaceStore((state) => state.engineReady);
  const optimizeScale = useWorkspaceStore((state) => state.optimizeScale);
  const buildCreasePattern = useWorkspaceStore((state) => state.buildCreasePattern);
  const workflowState = getCreasePatternWorkflowState({
    engineReady,
    status,
    edgeCount,
  });
  const showBuildAction =
    workflowState.canBuildCreasePattern || status === 'building_crease_pattern';

  if (showBuildAction) {
    return (
      <Button
        size="sm"
        variant="primary"
        disabled={!workflowState.canBuildCreasePattern}
        title={workflowState.buildCreasePatternReason}
        onClick={() => void buildCreasePattern()}
      >
        <Play size={13} />
        Build CP
      </Button>
    );
  }

  return (
    <Button
      size="sm"
      variant="primary"
      disabled={!workflowState.canOptimizeScale}
      title={workflowState.optimizeScaleReason}
      onClick={() => void optimizeScale()}
    >
      <Sparkles size={13} />
      Optimize Scale
    </Button>
  );
}
