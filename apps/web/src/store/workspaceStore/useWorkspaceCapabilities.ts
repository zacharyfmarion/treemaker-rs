import { useMemo } from 'react';
import { getWorkspaceCapabilities } from '../../lib/workspaceCapabilities';
import { useWorkspaceStore } from './store';

export function useWorkspaceCapabilities() {
  const documentMode = useWorkspaceStore((state) => state.documentMode);
  const engineReady = useWorkspaceStore((state) => state.engineReady);
  const status = useWorkspaceStore((state) => state.status);
  const edgeCount = useWorkspaceStore((state) => state.project.edges.length);
  const creaseCount = useWorkspaceStore((state) => state.project.creases.length);
  const facetCount = useWorkspaceStore((state) => state.project.facets.length);
  const hasImportedCreasePattern = useWorkspaceStore((state) => state.importedCreasePattern !== null);
  const hasSimulationModel = useWorkspaceStore((state) => state.foldArtifacts?.simulation_model != null);
  const historyPastCount = useWorkspaceStore((state) => state.historyPast.length);
  const historyFutureCount = useWorkspaceStore((state) => state.historyFuture.length);
  const clipboard = useWorkspaceStore((state) => state.clipboard);
  const selection = useWorkspaceStore((state) => state.selection);

  return useMemo(
    () =>
      getWorkspaceCapabilities({
        documentMode,
        engineReady,
        status,
        edgeCount,
        creaseCount,
        facetCount,
        hasImportedCreasePattern,
        hasSimulationModel,
        historyPastCount,
        historyFutureCount,
        clipboard,
        selection,
      }),
    [
      clipboard,
      creaseCount,
      documentMode,
      edgeCount,
      engineReady,
      facetCount,
      hasImportedCreasePattern,
      hasSimulationModel,
      historyFutureCount,
      historyPastCount,
      selection,
      status,
    ]
  );
}
