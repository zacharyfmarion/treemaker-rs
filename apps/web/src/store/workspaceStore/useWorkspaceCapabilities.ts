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
  const hasEditableCreasePattern = useWorkspaceStore((state) => state.oristudioCpDocument !== null);
  const hasImportedCreasePattern = useWorkspaceStore((state) => state.importedCreasePattern !== null);
  const hasSimulationModel = useWorkspaceStore((state) => state.foldArtifacts?.simulation_model != null);
  const oristudioCpSelectedLineCount = useWorkspaceStore(
    (state) => state.oristudioCpSelection.lines.length
  );
  const treeHistoryPastCount = useWorkspaceStore((state) => state.historyPast.length);
  const treeHistoryFutureCount = useWorkspaceStore((state) => state.historyFuture.length);
  const cpHistoryPastCount = useWorkspaceStore((state) => state.oristudioCpHistoryPast.length);
  const cpHistoryFutureCount = useWorkspaceStore((state) => state.oristudioCpHistoryFuture.length);
  const clipboard = useWorkspaceStore((state) => state.clipboard);
  const selection = useWorkspaceStore((state) => state.selection);
  const historyPastCount =
    documentMode === 'crease-pattern' ? cpHistoryPastCount : treeHistoryPastCount;
  const historyFutureCount =
    documentMode === 'crease-pattern' ? cpHistoryFutureCount : treeHistoryFutureCount;

  return useMemo(
    () =>
      getWorkspaceCapabilities({
        documentMode,
        engineReady,
        status,
        edgeCount,
        creaseCount,
        facetCount,
        hasEditableCreasePattern,
        hasImportedCreasePattern,
        hasSimulationModel,
        oristudioCpSelectedLineCount,
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
      hasEditableCreasePattern,
      hasImportedCreasePattern,
      hasSimulationModel,
      oristudioCpSelectedLineCount,
      historyFutureCount,
      historyPastCount,
      selection,
      status,
    ]
  );
}
