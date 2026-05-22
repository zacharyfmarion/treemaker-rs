import {
  getWorkspaceCapabilities,
  type WorkspaceCapabilities,
  type WorkspaceCapabilityInput,
} from '../../lib/workspaceCapabilities';
import type { WorkspaceState } from './types';

export function workspaceCapabilityInput(state: WorkspaceState): WorkspaceCapabilityInput {
  return {
    documentMode: state.documentMode,
    engineReady: state.engineReady,
    status: state.status,
    edgeCount: state.project.edges.length,
    creaseCount: state.project.creases.length,
    facetCount: state.project.facets.length,
    hasEditableCreasePattern: state.oristudioCpDocument !== null,
    hasImportedCreasePattern: state.importedCreasePattern !== null,
    hasSimulationModel: state.foldArtifacts?.simulation_model != null,
    historyPastCount:
      state.documentMode === 'crease-pattern'
        ? state.oristudioCpHistoryPast.length
        : state.historyPast.length,
    historyFutureCount:
      state.documentMode === 'crease-pattern'
        ? state.oristudioCpHistoryFuture.length
        : state.historyFuture.length,
    clipboard: state.clipboard,
    selection: state.selection,
  };
}

export function selectWorkspaceCapabilities(state: WorkspaceState): WorkspaceCapabilities {
  return getWorkspaceCapabilities(workspaceCapabilityInput(state));
}
