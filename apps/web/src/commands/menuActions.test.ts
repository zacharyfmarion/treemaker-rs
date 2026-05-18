import { describe, expect, it, vi } from 'vitest';
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
    },
    layout: {
      activatePanel: vi.fn(),
      resetLayout: vi.fn(),
    },
    fileService: createFileService('web'),
    quit: vi.fn(),
    about: vi.fn(),
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
    await expect(handle('cp.build')).resolves.toBe(true);
    await expect(handle('optimize.edges')).resolves.toBe(true);

    expect(deps.workspace.createNewProject).toHaveBeenCalledOnce();
    expect(deps.layout.activatePanel).toHaveBeenCalledWith('crease-pattern');
    expect(deps.layout.activatePanel).toHaveBeenCalledWith('simulator');
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

    expect(deps.workspace.undo).toHaveBeenCalledOnce();
    expect(deps.workspace.copySelection).toHaveBeenCalledOnce();
    expect(deps.workspace.selectAll).toHaveBeenCalledOnce();
    expect(deps.workspace.selectNone).toHaveBeenCalledOnce();
  });

  it('routes file commands through the selected file service', async () => {
    const deps = createDeps();

    await expect(createMenuActionHandler(deps)('file.saveAs')).resolves.toBe(true);
    expect(deps.workspace.saveProjectAs).toHaveBeenCalledWith(deps.fileService);

    await expect(createMenuActionHandler(deps)('file.exportFold')).resolves.toBe(true);
    expect(deps.workspace.exportFold).toHaveBeenCalledWith(deps.fileService);
  });

  it('returns false for unknown ids', async () => {
    await expect(createMenuActionHandler(createDeps())('unknown')).resolves.toBe(false);
  });
});
