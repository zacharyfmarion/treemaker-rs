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

export type MenuSubmenu = {
  type: 'submenu';
  label: string;
  items: MenuItemDef[];
};

export type MenuItemDef = MenuActionItem | MenuSeparator | MenuSubmenu;

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
        { type: 'action', id: 'file.exportCp', label: 'Export CP...' },
        { type: 'action', id: 'file.exportFold', label: 'Export FOLD...' },
        { type: 'action', id: 'file.exportSvg', label: 'Export SVG...' },
        { type: 'action', id: 'file.exportPng', label: 'Export PNG...' },
        { type: 'separator' },
        { type: 'action', id: 'file.settings', label: 'Settings', shortcut: `${mod}+,` },
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
        { type: 'action', id: 'edit.delete', label: 'Delete Selected', shortcut: 'Delete' },
        { type: 'separator' },
        {
          type: 'submenu',
          label: 'Select',
          items: [
            { type: 'action', id: 'edit.selectAll', label: 'Select All', shortcut: `${mod}+A` },
            { type: 'action', id: 'edit.deselectAll', label: 'Deselect All' },
            { type: 'action', id: 'edit.selectByIndex', label: 'Select By Index...' },
            { type: 'action', id: 'edit.selectMovableParts', label: 'Select Movable Parts' },
            { type: 'action', id: 'edit.selectCorridorFacets', label: 'Select Corridor Facets' },
          ],
        },
        {
          type: 'submenu',
          label: 'Node',
          items: [
            { type: 'action', id: 'edit.makeRoot', label: 'Make Root' },
            { type: 'action', id: 'edit.absorbNodes', label: 'Absorb Nodes' },
            { type: 'action', id: 'edit.absorbRedundantNodes', label: 'Absorb Redundant Nodes' },
            { type: 'separator' },
            { type: 'action', id: 'edit.perturbNodes', label: 'Perturb Nodes' },
            { type: 'action', id: 'edit.perturbAllNodes', label: 'Perturb All Nodes' },
          ],
        },
        {
          type: 'submenu',
          label: 'Edge',
          items: [
            { type: 'action', id: 'edit.splitEdge', label: 'Split Edge...' },
            { type: 'action', id: 'edit.setEdgeLength', label: 'Set Edge Length...' },
            { type: 'action', id: 'edit.scaleEdgeLengths', label: 'Scale Edge Lengths...' },
            { type: 'separator' },
            { type: 'action', id: 'edit.renormalizeToEdge', label: 'Renormalize To Edge' },
            { type: 'action', id: 'edit.renormalizeToUnitScale', label: 'Renormalize To Unit Scale' },
            { type: 'action', id: 'edit.absorbEdges', label: 'Absorb Edges' },
          ],
        },
        {
          type: 'submenu',
          label: 'Strain',
          items: [
            { type: 'action', id: 'edit.removeStrain', label: 'Remove Strain' },
            { type: 'action', id: 'edit.removeAllStrain', label: 'Remove All Strain' },
            { type: 'separator' },
            { type: 'action', id: 'edit.relieveStrain', label: 'Relieve Strain' },
            { type: 'action', id: 'edit.relieveAllStrain', label: 'Relieve All Strain' },
          ],
        },
        {
          type: 'submenu',
          label: 'Stubs',
          items: [
            { type: 'action', id: 'edit.addLargestStubForNodes', label: 'Add Largest Stub From Nodes' },
            { type: 'action', id: 'edit.addLargestStubForPoly', label: 'Add Largest Stub From Poly' },
            { type: 'separator' },
            { type: 'action', id: 'edit.triangulateTree', label: 'Triangulate Tree' },
          ],
        },
      ],
    },
    {
      label: 'View',
      items: [
        { type: 'action', id: 'view.design', label: 'Design' },
        { type: 'action', id: 'view.creasePattern', label: 'Crease Pattern' },
        { type: 'action', id: 'view.simulator', label: 'Simulator' },
        { type: 'action', id: 'view.foldedBase', label: 'Folded Base' },
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
      label: 'Crease Pattern',
      items: [
        { type: 'action', id: 'cp.foldedPreview', label: 'Show Folded Preview', shortcut: `${mod}+Shift+F` },
        { type: 'separator' },
        {
          type: 'submenu',
          label: 'Diagnostics',
          items: [
            { type: 'action', id: 'cp.checkCamv', label: 'Check CAMV', shortcut: `${mod}+Shift+M` },
            { type: 'action', id: 'cp.check1', label: 'Check 1' },
            { type: 'action', id: 'cp.check2', label: 'Check 2' },
            { type: 'action', id: 'cp.check3', label: 'Check 3' },
            { type: 'action', id: 'cp.check4', label: 'Check 4' },
          ],
        },
        {
          type: 'submenu',
          label: 'Repair',
          items: [
            { type: 'action', id: 'cp.fix1', label: 'Fix 1' },
            { type: 'action', id: 'cp.fix2', label: 'Fix 2' },
            { type: 'action', id: 'cp.fixInaccurate', label: 'Fix Inaccurate Creases' },
          ],
        },
        { type: 'separator' },
        { type: 'action', id: 'cp.deleteSelectedLines', label: 'Delete Selected Lines', shortcut: 'Delete' },
      ],
    },
    {
      label: 'Help',
      items: [
        { type: 'action', id: 'help.documentation', label: 'Ori Studio Help', shortcut: 'F1' },
        { type: 'separator' },
        { type: 'action', id: 'help.about', label: 'About Ori Studio' },
      ],
    },
  ];
}
