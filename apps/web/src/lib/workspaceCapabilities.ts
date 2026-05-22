import type { AppStatus, DocumentMode, Selection } from './sampleProject';

export type WorkspaceCapabilityId =
  | 'file.new'
  | 'file.open'
  | 'file.save'
  | 'file.saveAs'
  | 'file.exportV4'
  | 'file.exportCp'
  | 'file.exportFold'
  | 'file.exportSvg'
  | 'file.exportPng'
  | 'edit.undo'
  | 'edit.redo'
  | 'edit.cut'
  | 'edit.copy'
  | 'edit.paste'
  | 'edit.delete'
  | 'edit.selectAll'
  | 'edit.deselectAll'
  | 'edit.selectByIndex'
  | 'edit.selectMovableParts'
  | 'edit.selectCorridorFacets'
  | 'edit.makeRoot'
  | 'edit.splitEdge'
  | 'edit.setEdgeLength'
  | 'edit.scaleEdgeLengths'
  | 'edit.renormalizeToEdge'
  | 'edit.renormalizeToUnitScale'
  | 'edit.absorbNodes'
  | 'edit.absorbRedundantNodes'
  | 'edit.absorbEdges'
  | 'edit.perturbNodes'
  | 'edit.perturbAllNodes'
  | 'edit.removeStrain'
  | 'edit.removeAllStrain'
  | 'edit.relieveStrain'
  | 'edit.relieveAllStrain'
  | 'edit.addLargestStubForNodes'
  | 'edit.addLargestStubForPoly'
  | 'edit.triangulateTree'
  | 'view.design'
  | 'view.creasePattern'
  | 'view.simulator'
  | 'view.foldedBase'
  | 'view.conditions'
  | 'view.resetLayout'
  | 'optimize.scale'
  | 'optimize.edges'
  | 'optimize.strain'
  | 'cp.build'
  | 'simulator.refresh'
  | 'foldedBase.refresh';

export interface WorkspaceCapability {
  enabled: boolean;
  visible: boolean;
  label: string;
  reason: string;
}

export type WorkspaceCapabilities = Record<WorkspaceCapabilityId, WorkspaceCapability>;

export interface WorkspaceCapabilityInput {
  documentMode: DocumentMode;
  engineReady: boolean;
  status: AppStatus;
  edgeCount: number;
  creaseCount: number;
  facetCount: number;
  hasEditableCreasePattern: boolean;
  hasImportedCreasePattern: boolean;
  hasSimulationModel: boolean;
  historyPastCount: number;
  historyFutureCount: number;
  clipboard: unknown | null;
  selection: Selection;
}

export function getWorkspaceCapabilities(input: WorkspaceCapabilityInput): WorkspaceCapabilities {
  const treeMode = input.documentMode === 'tree';
  const creasePatternMode = input.documentMode === 'crease-pattern';
  const isBusy = isWorkspaceBusy(input.status);
  const hasTreeEdges = input.edgeCount > 0;
  const hasCreasePattern = input.creaseCount > 0 || input.facetCount > 0;
  const canOptimize =
    treeMode && input.engineReady && hasTreeEdges && !isBusy && input.status !== 'error';
  const canBuild =
    treeMode &&
    input.engineReady &&
    !isBusy &&
    (input.status === 'optimized' || input.status === 'crease_pattern_ready');
  const canExportTreeFold = treeMode && hasCreasePattern && !isBusy;
  const canExportImportedFold = creasePatternMode && input.hasImportedCreasePattern;
  const canSaveEditableCreasePattern = creasePatternMode && input.hasEditableCreasePattern;
  const canExportEditableCp = creasePatternMode && input.hasEditableCreasePattern;
  const canExportCreasePattern = hasCreasePattern && !isBusy;
  const hasSelection = selectionHasEditableParts(input.selection);
  const hasSelectedEdges = selectedEdgeCount(input.selection) > 0;
  const hasOneSelectedEdge = selectedEdgeCount(input.selection) === 1;
  const hasSelectedNodes = selectedNodeCount(input.selection) > 0;
  const hasOneSelectedNode = selectedNodeCount(input.selection) === 1;
  const buildLabel = hasCreasePattern ? 'Rebuild CP' : 'Build CP';
  const buildReason = hasCreasePattern ? 'Rebuild crease pattern' : 'Build crease pattern';

  return {
    'file.new': capability(!isBusy, 'New', isBusy ? busyReason(input.status) : 'Create a new Ori Studio project'),
    'file.open': capability(!isBusy, 'Open...', isBusy ? busyReason(input.status) : 'Open a project or crease pattern'),
    'file.save': capability(
      (treeMode || canSaveEditableCreasePattern) && !isBusy,
      'Save',
      treeMode
        ? busyOr('Save Ori Studio project', input.status)
        : canSaveEditableCreasePattern
          ? busyOr('Save editable crease pattern as CP', input.status)
          : 'Editable crease-pattern kernel is unavailable'
    ),
    'file.saveAs': capability(
      (treeMode || canSaveEditableCreasePattern) && !isBusy,
      'Save As...',
      treeMode
        ? busyOr('Save Ori Studio project as a new file', input.status)
        : canSaveEditableCreasePattern
          ? busyOr('Save editable crease pattern as a new CP file', input.status)
          : 'Editable crease-pattern kernel is unavailable'
    ),
    'file.exportV4': capability(
      treeMode && !isBusy,
      'Export TreeMaker 4...',
      treeMode ? busyOr('Export TreeMaker 4 project', input.status) : 'TreeMaker 4 export requires a tree document'
    ),
    'file.exportCp': capability(
      canExportEditableCp && !isBusy,
      'Export CP...',
      canExportEditableCp
        ? busyOr('Export editable crease pattern as CP', input.status)
        : 'Open an editable crease pattern before exporting CP'
    ),
    'file.exportFold': capability(
      (canExportTreeFold || canExportImportedFold) && !isBusy,
      'Export FOLD...',
      canExportTreeFold || canExportImportedFold
        ? busyOr('Export FOLD document', input.status)
        : treeMode
          ? 'Build a crease pattern before exporting FOLD'
          : 'Open a crease pattern before exporting FOLD'
    ),
    'file.exportSvg': capability(
      canExportCreasePattern,
      'Export SVG...',
      hasCreasePattern ? busyOr('Export crease pattern SVG', input.status) : 'No crease pattern to export'
    ),
    'file.exportPng': capability(
      canExportCreasePattern,
      'Export PNG...',
      hasCreasePattern ? busyOr('Export crease pattern PNG', input.status) : 'No crease pattern to export'
    ),
    'edit.undo': capability(
      treeMode && input.historyPastCount > 0 && !isBusy,
      'Undo',
      treeMode ? 'Undo the last tree edit' : 'Imported crease patterns are read-only'
    ),
    'edit.redo': capability(
      treeMode && input.historyFutureCount > 0 && !isBusy,
      'Redo',
      treeMode ? 'Redo the next tree edit' : 'Imported crease patterns are read-only'
    ),
    'edit.cut': capability(
      treeMode && hasSelection && !isBusy,
      'Cut',
      treeMode ? 'Cut selected tree parts' : 'Imported crease patterns are read-only'
    ),
    'edit.copy': capability(
      treeMode && hasSelection,
      'Copy',
      treeMode ? 'Copy selected tree parts' : 'Imported crease patterns are read-only'
    ),
    'edit.paste': capability(
      treeMode && input.clipboard !== null && !isBusy,
      'Paste',
      treeMode ? 'Paste copied tree parts' : 'Imported crease patterns are read-only'
    ),
    'edit.delete': capability(
      treeMode && hasSelection && !isBusy,
      'Delete Selected',
      treeMode ? 'Delete selected tree parts' : 'Imported crease patterns are read-only'
    ),
    'edit.selectAll': capability(true, 'Select All', 'Select visible document parts'),
    'edit.deselectAll': capability(true, 'Deselect All', 'Clear the current selection'),
    'edit.selectByIndex': capability(true, 'Select By Index...', 'Select a document part by its TreeMaker index'),
    'edit.selectMovableParts': capability(
      treeMode && !isBusy,
      'Select Movable Parts',
      treeMode ? busyOr('Select unpinned leaf nodes and movable edges', input.status) : 'Movable parts require a tree document'
    ),
    'edit.selectCorridorFacets': capability(
      input.facetCount > 0 && hasSelectedEdges,
      'Select Corridor Facets',
      input.facetCount > 0
        ? hasSelectedEdges
          ? 'Select facets in selected edge corridors'
          : 'Select one or more tree edges first'
        : 'Build a crease pattern before selecting corridor facets'
    ),
    'edit.makeRoot': capability(
      treeMode && hasOneSelectedNode && !isBusy,
      'Make Root',
      treeMode ? busyOr('Make selected node the root', input.status) : 'Root edits require a tree document'
    ),
    'edit.splitEdge': capability(
      treeMode && hasOneSelectedEdge && !isBusy,
      'Split Edge...',
      treeMode ? busyOr('Split selected edge', input.status) : 'Edge edits require a tree document'
    ),
    'edit.setEdgeLength': capability(
      treeMode && hasSelectedEdges && !isBusy,
      'Set Edge Length...',
      treeMode ? busyOr('Set selected edge lengths', input.status) : 'Edge edits require a tree document'
    ),
    'edit.scaleEdgeLengths': capability(
      treeMode && hasSelectedEdges && !isBusy,
      'Scale Edge Lengths...',
      treeMode ? busyOr('Scale selected edge lengths', input.status) : 'Edge edits require a tree document'
    ),
    'edit.renormalizeToEdge': capability(
      treeMode && hasOneSelectedEdge && !isBusy,
      'Renormalize To Edge',
      treeMode ? busyOr('Renormalize model to selected edge', input.status) : 'Edge edits require a tree document'
    ),
    'edit.renormalizeToUnitScale': capability(
      treeMode && input.edgeCount > 0 && !isBusy,
      'Renormalize To Unit Scale',
      treeMode ? busyOr('Renormalize model to unit scale', input.status) : 'Tree edits require a tree document'
    ),
    'edit.absorbNodes': capability(
      treeMode && hasSelectedNodes && !isBusy,
      'Absorb Nodes',
      treeMode ? busyOr('Absorb selected redundant nodes', input.status) : 'Node edits require a tree document'
    ),
    'edit.absorbRedundantNodes': capability(
      treeMode && input.edgeCount > 0 && !isBusy,
      'Absorb Redundant Nodes',
      treeMode ? busyOr('Absorb all degree-two internal nodes', input.status) : 'Node edits require a tree document'
    ),
    'edit.absorbEdges': capability(
      treeMode && hasSelectedEdges && !isBusy,
      'Absorb Edges',
      treeMode ? busyOr('Absorb selected edges', input.status) : 'Edge edits require a tree document'
    ),
    'edit.perturbNodes': capability(
      treeMode && hasSelectedNodes && !isBusy,
      'Perturb Nodes',
      treeMode ? busyOr('Perturb selected nodes', input.status) : 'Node edits require a tree document'
    ),
    'edit.perturbAllNodes': capability(
      treeMode && !isBusy,
      'Perturb All Nodes',
      treeMode ? busyOr('Perturb all nodes', input.status) : 'Node edits require a tree document'
    ),
    'edit.removeStrain': capability(
      treeMode && hasSelectedEdges && !isBusy,
      'Remove Strain',
      treeMode ? busyOr('Clear selected edge strain', input.status) : 'Edge edits require a tree document'
    ),
    'edit.removeAllStrain': capability(
      treeMode && input.edgeCount > 0 && !isBusy,
      'Remove All Strain',
      treeMode ? busyOr('Clear all edge strain', input.status) : 'Edge edits require a tree document'
    ),
    'edit.relieveStrain': capability(
      treeMode && hasSelectedEdges && !isBusy,
      'Relieve Strain',
      treeMode ? busyOr('Absorb selected strain into edge length', input.status) : 'Edge edits require a tree document'
    ),
    'edit.relieveAllStrain': capability(
      treeMode && input.edgeCount > 0 && !isBusy,
      'Relieve All Strain',
      treeMode ? busyOr('Absorb all strain into edge lengths', input.status) : 'Edge edits require a tree document'
    ),
    'edit.addLargestStubForNodes': capability(
      false,
      'Add Largest Stub From Nodes',
      'Stub finder port is pending'
    ),
    'edit.addLargestStubForPoly': capability(
      false,
      'Add Largest Stub From Poly',
      'Stub finder port is pending'
    ),
    'edit.triangulateTree': capability(false, 'Triangulate Tree', 'Stub finder triangulation port is pending'),
    'view.design': capability(true, 'Design', 'Show the design pane'),
    'view.creasePattern': capability(true, 'Crease Pattern', 'Show the crease pattern pane'),
    'view.simulator': capability(
      hasCreasePattern,
      'Simulator',
      hasCreasePattern ? 'Show the simulator pane' : 'Build or import a crease pattern before simulating'
    ),
    'view.foldedBase': capability(
      hasCreasePattern,
      'Folded Base',
      hasCreasePattern ? 'Show the folded base pane' : 'Build or import a crease pattern before viewing the folded base'
    ),
    'view.conditions': capability(true, 'Conditions', 'Show the conditions pane'),
    'view.resetLayout': capability(true, 'Reset Layout', 'Reset pane layout'),
    'optimize.scale': commandCapability(
      canOptimize,
      treeMode,
      'Optimize Scale',
      canOptimize ? 'Optimize Scale' : disabledOptimizeReason(input, isBusy, hasTreeEdges)
    ),
    'optimize.edges': commandCapability(
      canOptimize,
      treeMode,
      'Optimize Edges',
      canOptimize ? 'Optimize Edges' : disabledOptimizeReason(input, isBusy, hasTreeEdges)
    ),
    'optimize.strain': commandCapability(
      canOptimize,
      treeMode,
      'Optimize Strain',
      canOptimize ? 'Optimize Strain' : disabledOptimizeReason(input, isBusy, hasTreeEdges)
    ),
    'cp.build': commandCapability(
      canBuild,
      treeMode,
      buildLabel,
      canBuild ? buildReason : disabledBuildReason(input, isBusy, hasTreeEdges)
    ),
    'simulator.refresh': capability(
      treeMode && hasCreasePattern && !isBusy,
      'Refresh',
      treeMode ? busyOr('Refresh simulator model', input.status) : 'Imported crease patterns are prepared on import'
    ),
    'foldedBase.refresh': capability(
      treeMode && hasCreasePattern && !isBusy,
      'Refresh',
      treeMode ? busyOr('Refresh folded base', input.status) : 'Imported crease patterns are solved on import'
    ),
  };
}

export function getNextDocumentAction(
  capabilities: WorkspaceCapabilities
): WorkspaceCapabilityId | null {
  if (capabilities['cp.build'].visible && capabilities['cp.build'].enabled) return 'cp.build';
  if (capabilities['optimize.scale'].visible) return 'optimize.scale';
  return null;
}

export function isWorkspaceBusy(status: AppStatus): boolean {
  return status === 'loading_engine' || status === 'optimizing' || status === 'building_crease_pattern';
}

function capability(enabled: boolean, label: string, reason: string): WorkspaceCapability {
  return { enabled, visible: true, label, reason };
}

function commandCapability(
  enabled: boolean,
  visible: boolean,
  label: string,
  reason: string
): WorkspaceCapability {
  return { enabled, visible, label, reason };
}

function busyOr(reason: string, status: AppStatus): string {
  return isWorkspaceBusy(status) ? busyReason(status) : reason;
}

function busyReason(status: AppStatus): string {
  if (status === 'loading_engine') return 'Engine is still loading';
  if (status === 'optimizing') return 'Optimization is running';
  if (status === 'building_crease_pattern') return 'Crease pattern build is running';
  return 'Ori Studio is busy';
}

function disabledOptimizeReason(
  input: WorkspaceCapabilityInput,
  isBusy: boolean,
  hasTreeEdges: boolean
): string {
  if (input.documentMode !== 'tree') return 'Optimization requires an editable tree document';
  if (!input.engineReady || input.status === 'loading_engine') return 'Engine is still loading';
  if (isBusy) return busyReason(input.status);
  if (!hasTreeEdges) return 'Add at least one tree edge before optimizing';
  if (input.status === 'error') return 'Resolve the current engine error before optimizing';
  return 'Optimization is unavailable';
}

function disabledBuildReason(
  input: WorkspaceCapabilityInput,
  isBusy: boolean,
  hasTreeEdges: boolean
): string {
  if (input.documentMode !== 'tree') return 'Build CP requires an editable tree document';
  if (!input.engineReady || input.status === 'loading_engine') return 'Engine is still loading';
  if (isBusy) return busyReason(input.status);
  if (input.status === 'error') return 'Resolve the current engine error before building the crease pattern';
  if (!hasTreeEdges) return 'Add tree edges, then optimize before building the crease pattern';
  return 'Optimize Scale before building the crease pattern';
}

function selectedEdgeCount(selection: Selection): number {
  if (selection.kind === 'edge') return 1;
  if (selection.kind === 'multi') return selection.edges.length;
  return 0;
}

function selectedNodeCount(selection: Selection): number {
  if (selection.kind === 'node') return 1;
  if (selection.kind === 'multi') return selection.nodes.length;
  return 0;
}

function selectionHasEditableParts(selection: Selection): boolean {
  if (selection.kind === 'tree') return false;
  if (selection.kind === 'multi') {
    return (
      selection.nodes.length +
        selection.edges.length +
        selection.paths.length +
        selection.conditions.length >
      0
    );
  }
  return selection.kind === 'node' || selection.kind === 'edge' || selection.kind === 'path' || selection.kind === 'condition';
}
