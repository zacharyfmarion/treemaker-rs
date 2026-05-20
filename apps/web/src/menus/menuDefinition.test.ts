import { describe, expect, it } from 'vitest';
import { isMenuActionId } from '../commands/menuActions';
import { getMenuBarDef, type MenuItemDef } from './menuDefinition';

function actionIdsFor(items: MenuItemDef[]) {
  return items.flatMap((item): string[] => {
    if (item.type === 'action') return [item.id];
    if (item.type === 'submenu') return actionIdsFor(item.items);
    return [];
  });
}

describe('web menu definition', () => {
  it('uses command ids supported by the shared menu dispatcher', () => {
    const actionIds = getMenuBarDef().flatMap((menu) => actionIdsFor(menu.items));

    expect(actionIds.length).toBeGreaterThan(0);
    expect(actionIds.every((id) => isMenuActionId(id))).toBe(true);
  });

  it('keeps quit out of the web menu surface', () => {
    const actionIds = getMenuBarDef().flatMap((menu) => actionIdsFor(menu.items));

    expect(actionIds).not.toContain('app.quit');
    expect(actionIds).not.toContain('app.about');
  });

  it('groups extended edit tools into compact submenus', () => {
    const editMenu = getMenuBarDef().find((menu) => menu.label === 'Edit');
    const submenuLabels = editMenu?.items.flatMap((item) =>
      item.type === 'submenu' ? [item.label] : []
    );

    expect(submenuLabels).toEqual(['Select', 'Node', 'Edge', 'Strain', 'Stubs']);
  });

  it('exposes documentation and about from the Help menu', () => {
    const helpMenu = getMenuBarDef().find((menu) => menu.label === 'Help');
    const actionIds = helpMenu ? actionIdsFor(helpMenu.items) : undefined;

    expect(actionIds).toEqual(['help.documentation', 'help.about']);
  });
});
