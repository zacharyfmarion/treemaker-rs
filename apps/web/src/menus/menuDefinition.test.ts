import { describe, expect, it } from 'vitest';
import { isMenuActionId } from '../commands/menuActions';
import { getMenuBarDef } from './menuDefinition';

describe('web menu definition', () => {
  it('uses command ids supported by the shared menu dispatcher', () => {
    const actionIds = getMenuBarDef().flatMap((menu) =>
      menu.items.flatMap((item) => (item.type === 'action' ? [item.id] : []))
    );

    expect(actionIds.length).toBeGreaterThan(0);
    expect(actionIds.every((id) => isMenuActionId(id))).toBe(true);
  });

  it('keeps quit out of the web menu surface', () => {
    const actionIds = getMenuBarDef().flatMap((menu) =>
      menu.items.flatMap((item) => (item.type === 'action' ? [item.id] : []))
    );

    expect(actionIds).not.toContain('app.quit');
    expect(actionIds).not.toContain('app.about');
  });

  it('exposes documentation and about from the Help menu', () => {
    const helpMenu = getMenuBarDef().find((menu) => menu.label === 'Help');
    const actionIds = helpMenu?.items.flatMap((item) =>
      item.type === 'action' ? [item.id] : []
    );

    expect(actionIds).toEqual(['help.documentation', 'help.about']);
  });
});
