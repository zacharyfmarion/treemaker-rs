import { describe, expect, it } from 'vitest';
import {
  DEFAULT_ORISTUDIO_CP_ACTION_ID,
  ORISTUDIO_CP_ACTION_GROUPS,
  ORISTUDIO_CP_LINE_TYPE_ACTIONS,
  cpActionById,
  cpRailActions,
} from './oristudioCpActions';
import { ORISTUDIO_CP_COMMANDS } from './oristudioCpCommands';

describe('oristudio CP action registry', () => {
  it('adds Oriedita line type actions before command actions', () => {
    expect(ORISTUDIO_CP_ACTION_GROUPS.map((group) => group.id).slice(0, 3)).toEqual([
      'line-type',
      'select-edit',
      'draw',
    ]);
    expect(ORISTUDIO_CP_LINE_TYPE_ACTIONS.map((action) => action.railLabel)).toEqual([
      'M',
      'V',
      'E',
      'A',
    ]);
    expect(ORISTUDIO_CP_LINE_TYPE_ACTIONS.map((action) => action.lineColor)).toEqual([
      'Red1',
      'Blue2',
      'Black0',
      'Cyan3',
    ]);
  });

  it('keeps every operation-backed command reachable through an action', () => {
    const operationActionIds = new Set(
      cpRailActions()
        .filter((action) => action.kind === 'command')
        .map((action) => action.operationId)
    );

    for (const command of ORISTUDIO_CP_COMMANDS) {
      if (command.placement === 'hidden-ui-only') continue;
      if (command.placement === 'menu' || command.placement === 'palette') continue;
      expect(operationActionIds.has(command.operationId), command.operationId).toBe(true);
    }
  });

  it('models draw crease as a repeatable drag-line action', () => {
    expect(cpActionById(DEFAULT_ORISTUDIO_CP_ACTION_ID)).toMatchObject({
      kind: 'command',
      operationId: 'CreaseSelect',
      inputMode: 'drag-box',
      repeatable: true,
    });

    expect(cpActionById('cp.action.draw-crease')).toMatchObject({
      kind: 'command',
      operationId: 'DrawCreaseFree',
      inputMode: 'drag-line',
      repeatable: true,
      toolSteps: ['Drag crease endpoint'],
      upstreamAction: 'drawCreaseFreeAction',
    });

    expect(cpActionById('cp.action.draw-auxiliary-line')).toMatchObject({
      kind: 'command',
      operationId: 'DrawCreaseFree',
      inputMode: 'drag-line',
      lineInputMode: 'aux-line',
      uiStatus: 'not-implemented',
    });
  });
});
