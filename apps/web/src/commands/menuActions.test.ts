import { describe, expect, it, vi } from 'vitest';
import { getWorkspaceCapabilities } from '../lib/workspaceCapabilities';
import { createFileService } from '../platform/fileService';
import { createMenuActionHandler, isMenuActionId } from './menuActions';

function createDeps() {
  return {
    workspace: {
      createNewProject: vi.fn().mockResolvedValue(undefined),
      openProject: vi.fn().mockResolvedValue(true),
      saveProject: vi.fn().mockResolvedValue(true),
      saveProjectAs: vi.fn().mockResolvedValue(true),
      exportV4: vi.fn().mockResolvedValue(true),
      exportFold: vi.fn().mockResolvedValue(true),
      exportSvg: vi.fn().mockResolvedValue(true),
      exportPng: vi.fn().mockResolvedValue(true),
      undo: vi.fn().mockResolvedValue(undefined),
      redo: vi.fn().mockResolvedValue(undefined),
      cutSelection: vi.fn().mockResolvedValue(undefined),
      copySelection: vi.fn(),
      pasteClipboard: vi.fn().mockResolvedValue(undefined),
      deleteSelection: vi.fn().mockResolvedValue(undefined),
      optimizeScale: vi.fn().mockResolvedValue(undefined),
      optimizeEdges: vi.fn().mockResolvedValue(undefined),
      optimizeStrain: vi.fn().mockResolvedValue(undefined),
      buildCreasePattern: vi.fn().mockResolvedValue(undefined),
      select: vi.fn(),
      selectAll: vi.fn(),
      selectNone: vi.fn(),
      selectMovableParts: vi.fn(),
      selectCorridorFacets: vi.fn(),
      makeSelectedNodeRoot: vi.fn().mockResolvedValue(undefined),
      splitSelectedEdge: vi.fn().mockResolvedValue(undefined),
      setSelectedEdgeLengths: vi.fn().mockResolvedValue(undefined),
      scaleSelectedEdgeLengths: vi.fn().mockResolvedValue(undefined),
      renormalizeToSelectedEdge: vi.fn().mockResolvedValue(undefined),
      renormalizeToUnitScale: vi.fn().mockResolvedValue(undefined),
      absorbSelectedNodes: vi.fn().mockResolvedValue(undefined),
      absorbRedundantNodes: vi.fn().mockResolvedValue(undefined),
      absorbSelectedEdges: vi.fn().mockResolvedValue(undefined),
      perturbSelectedNodes: vi.fn().mockResolvedValue(undefined),
      perturbAllNodes: vi.fn().mockResolvedValue(undefined),
      removeSelectionStrain: vi.fn().mockResolvedValue(undefined),
      removeAllStrain: vi.fn().mockResolvedValue(undefined),
      relieveSelectionStrain: vi.fn().mockResolvedValue(undefined),
      relieveAllStrain: vi.fn().mockResolvedValue(undefined),
      addLargestStubForSelectedNodes: vi.fn().mockResolvedValue(undefined),
      addLargestStubForSelectedPoly: vi.fn().mockResolvedValue(undefined),
      triangulateTree: vi.fn().mockResolvedValue(undefined),
    },
    layout: {
      activatePanel: vi.fn(),
      resetLayout: vi.fn(),
    },
    fileService: createFileService('web'),
    quit: vi.fn(),
    help: vi.fn(),
    about: vi.fn(),
    settings: vi.fn(),
    selectByIndex: vi.fn(),
    requestPositiveNumber: vi.fn().mockResolvedValue(0.5),
  };
}

describe('menu actions', () => {
  it('recognizes shared command ids', () => {
    expect(isMenuActionId('file.new')).toBe(true);
    expect(isMenuActionId('cp.build')).toBe(true);
    expect(isMenuActionId('not.real')).toBe(false);
  });

  it('dispatches document and layout commands', async () => {
    const deps = createDeps();
    const handle = createMenuActionHandler(deps);

    await expect(handle('file.new')).resolves.toBe(true);
    await expect(handle('view.creasePattern')).resolves.toBe(true);
    await expect(handle('view.simulator')).resolves.toBe(true);
    await expect(handle('view.foldedBase')).resolves.toBe(true);
    await expect(handle('file.settings')).resolves.toBe(true);
    await expect(handle('help.documentation')).resolves.toBe(true);
    await expect(handle('help.about')).resolves.toBe(true);
    await expect(handle('app.about')).resolves.toBe(true);
    await expect(handle('cp.build')).resolves.toBe(true);
    await expect(handle('optimize.edges')).resolves.toBe(true);

    expect(deps.workspace.createNewProject).toHaveBeenCalledOnce();
    expect(deps.layout.activatePanel).toHaveBeenCalledWith('crease-pattern');
    expect(deps.layout.activatePanel).toHaveBeenCalledWith('simulator');
    expect(deps.layout.activatePanel).toHaveBeenCalledWith('folded-base');
    expect(deps.settings).toHaveBeenCalledOnce();
    expect(deps.help).toHaveBeenCalledOnce();
    expect(deps.about).toHaveBeenCalledTimes(2);
    expect(deps.workspace.buildCreasePattern).toHaveBeenCalledOnce();
    expect(deps.workspace.optimizeEdges).toHaveBeenCalledOnce();
  });

  it('dispatches edit commands through workspace actions', async () => {
    const deps = createDeps();
    const handle = createMenuActionHandler(deps);

    await expect(handle('edit.undo')).resolves.toBe(true);
    await expect(handle('edit.copy')).resolves.toBe(true);
    await expect(handle('edit.selectAll')).resolves.toBe(true);
    await expect(handle('edit.deselectAll')).resolves.toBe(true);
    await expect(handle('edit.selectByIndex')).resolves.toBe(true);
    await expect(handle('edit.selectMovableParts')).resolves.toBe(true);
    await expect(handle('edit.selectCorridorFacets')).resolves.toBe(true);
    await expect(handle('edit.makeRoot')).resolves.toBe(true);
    await expect(handle('edit.renormalizeToEdge')).resolves.toBe(true);
    await expect(handle('edit.absorbNodes')).resolves.toBe(true);
    await expect(handle('edit.perturbAllNodes')).resolves.toBe(true);
    await expect(handle('edit.relieveAllStrain')).resolves.toBe(true);
    await expect(handle('edit.triangulateTree')).resolves.toBe(true);

    expect(deps.workspace.undo).toHaveBeenCalledOnce();
    expect(deps.workspace.copySelection).toHaveBeenCalledOnce();
    expect(deps.workspace.selectAll).toHaveBeenCalledOnce();
    expect(deps.workspace.selectNone).toHaveBeenCalledOnce();
    expect(deps.selectByIndex).toHaveBeenCalledOnce();
    expect(deps.workspace.selectMovableParts).toHaveBeenCalledOnce();
    expect(deps.workspace.selectCorridorFacets).toHaveBeenCalledOnce();
    expect(deps.workspace.makeSelectedNodeRoot).toHaveBeenCalledOnce();
    expect(deps.workspace.renormalizeToSelectedEdge).toHaveBeenCalledOnce();
    expect(deps.workspace.absorbSelectedNodes).toHaveBeenCalledOnce();
    expect(deps.workspace.perturbAllNodes).toHaveBeenCalledOnce();
    expect(deps.workspace.relieveAllStrain).toHaveBeenCalledOnce();
    expect(deps.workspace.triangulateTree).toHaveBeenCalledOnce();
  });

  it('requests in-app numeric values for parameterized edit commands', async () => {
    const deps = createDeps();
    deps.requestPositiveNumber
      .mockResolvedValueOnce(0.25)
      .mockResolvedValueOnce(2)
      .mockResolvedValueOnce(1.5);
    const handle = createMenuActionHandler(deps);

    await expect(handle('edit.splitEdge')).resolves.toBe(true);
    await expect(handle('edit.setEdgeLength')).resolves.toBe(true);
    await expect(handle('edit.scaleEdgeLengths')).resolves.toBe(true);

    expect(deps.requestPositiveNumber).toHaveBeenNthCalledWith(
      1,
      expect.objectContaining({ title: 'Split Edge', initialValue: '0.5' })
    );
    expect(deps.workspace.splitSelectedEdge).toHaveBeenCalledWith(0.25);
    expect(deps.workspace.setSelectedEdgeLengths).toHaveBeenCalledWith(2);
    expect(deps.workspace.scaleSelectedEdgeLengths).toHaveBeenCalledWith(1.5);
  });

  it('cancels parameterized edit commands when the in-app number modal is dismissed', async () => {
    const deps = createDeps();
    deps.requestPositiveNumber.mockResolvedValueOnce(null);

    await expect(createMenuActionHandler(deps)('edit.splitEdge')).resolves.toBe(false);

    expect(deps.workspace.splitSelectedEdge).not.toHaveBeenCalled();
  });

  it('routes file commands through the selected file service', async () => {
    const deps = createDeps();

    await expect(createMenuActionHandler(deps)('file.saveAs')).resolves.toBe(true);
    expect(deps.workspace.saveProjectAs).toHaveBeenCalledWith(deps.fileService);

    await expect(createMenuActionHandler(deps)('file.exportFold')).resolves.toBe(true);
    expect(deps.workspace.exportFold).toHaveBeenCalledWith(deps.fileService);
  });

  it('does not dispatch disabled capabilities', async () => {
    const deps = {
      ...createDeps(),
      capabilities: () =>
        getWorkspaceCapabilities({
        documentMode: 'crease-pattern',
        engineReady: true,
        status: 'crease_pattern_ready',
        edgeCount: 0,
        creaseCount: 4,
        facetCount: 1,
        hasImportedCreasePattern: true,
        hasSimulationModel: true,
        historyPastCount: 0,
        historyFutureCount: 0,
        clipboard: null,
        selection: { kind: 'tree' },
        }),
    };

    await expect(createMenuActionHandler(deps)('cp.build')).resolves.toBe(false);

    expect(deps.workspace.buildCreasePattern).not.toHaveBeenCalled();
  });

  it('returns false for unknown ids', async () => {
    await expect(createMenuActionHandler(createDeps())('unknown')).resolves.toBe(false);
  });
});
