import type { AppStatus, DocumentMode, Selection } from './sampleProject';

export type WorkspaceCapabilityId =
  | 'file.new'
  | 'file.open'
  | 'file.save'
  | 'file.saveAs'
  | 'file.exportV4'
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
  const canExportCreasePattern = hasCreasePattern && !isBusy;
  const hasSelection = selectionHasEditableParts(input.selection);
  const buildLabel = hasCreasePattern ? 'Rebuild CP' : 'Build CP';
  const buildReason = hasCreasePattern ? 'Rebuild crease pattern' : 'Build crease pattern';

  return {
    'file.new': capability(!isBusy, 'New', isBusy ? busyReason(input.status) : 'Create a new Ori Studio project'),
    'file.open': capability(!isBusy, 'Open...', isBusy ? busyReason(input.status) : 'Open a project or crease pattern'),
    'file.save': capability(
      treeMode && !isBusy,
      'Save',
      treeMode ? busyOr('Save Ori Studio project', input.status) : 'Imported crease patterns are exported, not saved as Ori Studio projects'
    ),
    'file.saveAs': capability(
      treeMode && !isBusy,
      'Save As...',
      treeMode ? busyOr('Save Ori Studio project as a new file', input.status) : 'Imported crease patterns are exported, not saved as Ori Studio projects'
    ),
    'file.exportV4': capability(
      treeMode && !isBusy,
      'Export TreeMaker 4...',
      treeMode ? busyOr('Export TreeMaker 4 project', input.status) : 'TreeMaker 4 export requires a tree document'
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
