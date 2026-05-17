import { getFileService, type FileCommand, type FileService } from '../platform/fileService';
import { useLayoutStore } from '../store/layoutStore';
import { useWorkspaceStore } from '../store/workspaceStore';

export const MENU_ACTION_IDS = [
  'app.quit',
  'file.new',
  'file.open',
  'file.save',
  'file.saveAs',
  'file.exportV4',
  'file.exportSvg',
  'file.exportPng',
  'edit.delete',
  'edit.deselectAll',
  'view.design',
  'view.creasePattern',
  'view.resetLayout',
  'optimize.scale',
  'cp.build',
  'help.about',
] as const;

export type MenuActionId = (typeof MENU_ACTION_IDS)[number];

export interface WorkspaceCommands {
  createNewProject(): Promise<void>;
  deleteSelection(): Promise<void>;
  optimizeScale(): Promise<void>;
  buildCreasePattern(): Promise<void>;
  select(selection: { kind: 'tree' }): void;
}

export interface LayoutCommands {
  activatePanel(id: string): void;
  resetLayout(): void;
}

export interface MenuActionDependencies {
  workspace: WorkspaceCommands;
  layout: LayoutCommands;
  fileService: FileService;
  quit?: () => void;
  about?: () => void;
}

const FILE_ACTIONS: Partial<Record<MenuActionId, FileCommand>> = {
  'file.open': 'openProject',
  'file.save': 'saveProject',
  'file.saveAs': 'saveProjectAs',
  'file.exportV4': 'exportV4',
  'file.exportSvg': 'exportSvg',
  'file.exportPng': 'exportPng',
};

export function isMenuActionId(id: string): id is MenuActionId {
  return (MENU_ACTION_IDS as readonly string[]).includes(id);
}

function logFileResult(message: string): void {
  console.info(message);
}

export function createMenuActionHandler(deps: MenuActionDependencies) {
  return async (id: string): Promise<boolean> => {
    if (!isMenuActionId(id)) {
      console.warn(`Unknown menu action: ${id}`);
      return false;
    }

    const fileCommand = FILE_ACTIONS[id];
    if (fileCommand) {
      const result = await deps.fileService.run(fileCommand);
      logFileResult(result.message);
      return result.status === 'handled';
    }

    switch (id) {
      case 'app.quit':
        deps.quit?.();
        return true;
      case 'file.new':
        await deps.workspace.createNewProject();
        return true;
      case 'edit.delete':
        await deps.workspace.deleteSelection();
        return true;
      case 'edit.deselectAll':
        deps.workspace.select({ kind: 'tree' });
        return true;
      case 'view.design':
        deps.layout.activatePanel('design');
        return true;
      case 'view.creasePattern':
        deps.layout.activatePanel('crease-pattern');
        return true;
      case 'view.resetLayout':
        deps.layout.resetLayout();
        return true;
      case 'optimize.scale':
        await deps.workspace.optimizeScale();
        return true;
      case 'cp.build':
        await deps.workspace.buildCreasePattern();
        return true;
      case 'help.about':
        deps.about?.();
        return true;
    }

    return false;
  };
}

export function handleMenuAction(id: string): Promise<boolean> {
  return createMenuActionHandler({
    workspace: useWorkspaceStore.getState(),
    layout: useLayoutStore.getState(),
    fileService: getFileService(),
    about: () => {
      console.info('TreeMaker web and desktop shell');
    },
  })(id);
}
