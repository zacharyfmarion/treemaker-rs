import type { MenuActionId } from '../commands/menuActions';

export type MenuActionItem = {
  type: 'action';
  id: MenuActionId;
  label: string;
  shortcut?: string;
};

export type MenuSeparator = {
  type: 'separator';
};

export type MenuItemDef = MenuActionItem | MenuSeparator;

export type MenuDef = {
  label: string;
  items: MenuItemDef[];
};

function modKey(): string {
  const isMac =
    typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform);
  return isMac ? 'Cmd' : 'Ctrl';
}

export function getMenuBarDef(): MenuDef[] {
  const mod = modKey();

  return [
    {
      label: 'File',
      items: [
        { type: 'action', id: 'file.new', label: 'New', shortcut: `${mod}+N` },
        { type: 'action', id: 'file.open', label: 'Open...', shortcut: `${mod}+O` },
        { type: 'separator' },
        { type: 'action', id: 'file.save', label: 'Save', shortcut: `${mod}+S` },
        { type: 'action', id: 'file.saveAs', label: 'Save As...', shortcut: `${mod}+Shift+S` },
        { type: 'separator' },
        { type: 'action', id: 'file.exportV4', label: 'Export TreeMaker 4...' },
        { type: 'action', id: 'file.exportFold', label: 'Export FOLD...' },
        { type: 'action', id: 'file.exportSvg', label: 'Export SVG...' },
        { type: 'action', id: 'file.exportPng', label: 'Export PNG...' },
      ],
    },
    {
      label: 'Edit',
      items: [
        { type: 'action', id: 'edit.undo', label: 'Undo', shortcut: `${mod}+Z` },
        { type: 'action', id: 'edit.redo', label: 'Redo', shortcut: `${mod}+Shift+Z` },
        { type: 'separator' },
        { type: 'action', id: 'edit.cut', label: 'Cut', shortcut: `${mod}+X` },
        { type: 'action', id: 'edit.copy', label: 'Copy', shortcut: `${mod}+C` },
        { type: 'action', id: 'edit.paste', label: 'Paste', shortcut: `${mod}+V` },
        { type: 'separator' },
        { type: 'action', id: 'edit.selectAll', label: 'Select All', shortcut: `${mod}+A` },
        { type: 'action', id: 'edit.deselectAll', label: 'Deselect All' },
        { type: 'action', id: 'edit.delete', label: 'Delete Selected', shortcut: 'Delete' },
      ],
    },
    {
      label: 'View',
      items: [
        { type: 'action', id: 'view.design', label: 'Design' },
        { type: 'action', id: 'view.creasePattern', label: 'Crease Pattern' },
        { type: 'action', id: 'view.simulator', label: 'Simulator' },
        { type: 'action', id: 'view.conditions', label: 'Conditions' },
        { type: 'separator' },
        { type: 'action', id: 'view.resetLayout', label: 'Reset Layout' },
      ],
    },
    {
      label: 'Design',
      items: [
        { type: 'action', id: 'optimize.scale', label: 'Optimize Scale', shortcut: `${mod}+R` },
        { type: 'action', id: 'optimize.edges', label: 'Optimize Edges' },
        { type: 'action', id: 'optimize.strain', label: 'Optimize Strain' },
        { type: 'separator' },
        { type: 'action', id: 'cp.build', label: 'Build Crease Pattern', shortcut: `${mod}+B` },
      ],
    },
    {
      label: 'Help',
      items: [{ type: 'action', id: 'help.about', label: 'About TreeMaker' }],
    },
  ];
}
