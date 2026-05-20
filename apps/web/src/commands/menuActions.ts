import { getFileService, type FileCommand, type FileService } from '../platform/fileService';
import { useHelpStore } from '../store/helpStore';
import { useLayoutStore } from '../store/layoutStore';
import { useSelectionUiStore } from '../store/selectionUiStore';
import { useSettingsStore } from '../store/settingsStore';
import { useWorkspaceStore } from '../store/workspaceStore';
import { selectWorkspaceCapabilities } from '../store/workspaceStore/capabilities';
import type { WorkspaceCapabilities, WorkspaceCapabilityId } from '../lib/workspaceCapabilities';

export const MENU_ACTION_IDS = [
  'app.about',
  'app.quit',
  'file.new',
  'file.open',
  'file.save',
  'file.saveAs',
  'file.settings',
  'file.exportV4',
  'file.exportFold',
  'file.exportSvg',
  'file.exportPng',
  'edit.undo',
  'edit.redo',
  'edit.cut',
  'edit.copy',
  'edit.paste',
  'edit.delete',
  'edit.selectAll',
  'edit.deselectAll',
  'edit.selectByIndex',
  'edit.selectMovableParts',
  'edit.selectCorridorFacets',
  'view.design',
  'view.creasePattern',
  'view.simulator',
  'view.foldedBase',
  'view.conditions',
  'view.resetLayout',
  'optimize.scale',
  'optimize.edges',
  'optimize.strain',
  'cp.build',
  'help.documentation',
  'help.about',
] as const;

export type MenuActionId = (typeof MENU_ACTION_IDS)[number];

export interface WorkspaceCommands {
  createNewProject(): Promise<void>;
  openProject(fileService?: FileService): Promise<boolean>;
  saveProject(fileService?: FileService): Promise<boolean>;
  saveProjectAs(fileService?: FileService): Promise<boolean>;
  exportV4(fileService?: FileService): Promise<boolean>;
  exportFold(fileService?: FileService): Promise<boolean>;
  exportSvg(fileService?: FileService): Promise<boolean>;
  exportPng(fileService?: FileService): Promise<boolean>;
  undo(): Promise<void>;
  redo(): Promise<void>;
  cutSelection(): Promise<void>;
  copySelection(): void;
  pasteClipboard(): Promise<void>;
  deleteSelection(): Promise<void>;
  optimizeScale(): Promise<void>;
  optimizeEdges(): Promise<void>;
  optimizeStrain(): Promise<void>;
  buildCreasePattern(): Promise<void>;
  select(selection: { kind: 'tree' }): void;
  selectAll(): void;
  selectNone(): void;
  selectMovableParts(): void;
  selectCorridorFacets(): void;
}

export interface LayoutCommands {
  activatePanel(id: string): void;
  resetLayout(): void;
}

export interface MenuActionDependencies {
  workspace: WorkspaceCommands;
  layout: LayoutCommands;
  fileService: FileService;
  capabilities?: () => WorkspaceCapabilities;
  quit?: () => void;
  help?: () => void;
  about?: () => void;
  settings?: () => void;
  selectByIndex?: () => void;
}

const FILE_ACTIONS: Partial<Record<MenuActionId, FileCommand>> = {
  'file.open': 'openProject',
  'file.save': 'saveProject',
  'file.saveAs': 'saveProjectAs',
  'file.exportV4': 'exportV4',
  'file.exportFold': 'exportFold',
  'file.exportSvg': 'exportSvg',
  'file.exportPng': 'exportPng',
};

export function isMenuActionId(id: string): id is MenuActionId {
  return (MENU_ACTION_IDS as readonly string[]).includes(id);
}

export function createMenuActionHandler(deps: MenuActionDependencies) {
  return async (id: string): Promise<boolean> => {
    if (!isMenuActionId(id)) {
      console.warn(`Unknown menu action: ${id}`);
      return false;
    }

    const capability = deps.capabilities?.()[id as WorkspaceCapabilityId];
    if (capability && !capability.enabled) {
      console.info(`Menu action disabled: ${id}: ${capability.reason}`);
      return false;
    }

    const fileCommand = FILE_ACTIONS[id];
    if (fileCommand) {
      switch (fileCommand) {
        case 'openProject':
          return deps.workspace.openProject(deps.fileService);
        case 'saveProject':
          return deps.workspace.saveProject(deps.fileService);
        case 'saveProjectAs':
          return deps.workspace.saveProjectAs(deps.fileService);
        case 'exportV4':
          return deps.workspace.exportV4(deps.fileService);
        case 'exportFold':
          return deps.workspace.exportFold(deps.fileService);
        case 'exportSvg':
          return deps.workspace.exportSvg(deps.fileService);
        case 'exportPng':
          return deps.workspace.exportPng(deps.fileService);
      }
    }

    switch (id) {
      case 'app.about':
        deps.about?.();
        return true;
      case 'app.quit':
        deps.quit?.();
        return true;
      case 'file.new':
        await deps.workspace.createNewProject();
        return true;
      case 'file.settings':
        deps.settings?.();
        return true;
      case 'edit.undo':
        await deps.workspace.undo();
        return true;
      case 'edit.redo':
        await deps.workspace.redo();
        return true;
      case 'edit.cut':
        await deps.workspace.cutSelection();
        return true;
      case 'edit.copy':
        deps.workspace.copySelection();
        return true;
      case 'edit.paste':
        await deps.workspace.pasteClipboard();
        return true;
      case 'edit.delete':
        await deps.workspace.deleteSelection();
        return true;
      case 'edit.selectAll':
        deps.workspace.selectAll();
        return true;
      case 'edit.deselectAll':
        deps.workspace.selectNone();
        return true;
      case 'edit.selectByIndex':
        deps.selectByIndex?.();
        return true;
      case 'edit.selectMovableParts':
        deps.workspace.selectMovableParts();
        return true;
      case 'edit.selectCorridorFacets':
        deps.workspace.selectCorridorFacets();
        return true;
      case 'view.design':
        deps.layout.activatePanel('design');
        return true;
      case 'view.creasePattern':
        deps.layout.activatePanel('crease-pattern');
        return true;
      case 'view.simulator':
        deps.layout.activatePanel('simulator');
        return true;
      case 'view.foldedBase':
        deps.layout.activatePanel('folded-base');
        return true;
      case 'view.conditions':
        deps.layout.activatePanel('conditions');
        return true;
      case 'view.resetLayout':
        deps.layout.resetLayout();
        return true;
      case 'optimize.scale':
        await deps.workspace.optimizeScale();
        return true;
      case 'optimize.edges':
        await deps.workspace.optimizeEdges();
        return true;
      case 'optimize.strain':
        await deps.workspace.optimizeStrain();
        return true;
      case 'cp.build':
        await deps.workspace.buildCreasePattern();
        return true;
      case 'help.documentation':
        deps.help?.();
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
    capabilities: () => selectWorkspaceCapabilities(useWorkspaceStore.getState()),
    settings: () => {
      useSettingsStore.getState().openSettings();
    },
    help: () => {
      useHelpStore.getState().openGuide();
    },
    about: () => {
      useHelpStore.getState().openAbout();
    },
    selectByIndex: () => {
      useSelectionUiStore.getState().openSelectByIndex();
    },
  })(id);
}
