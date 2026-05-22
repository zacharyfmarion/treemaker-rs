import { describe, expect, it } from 'vitest';
import {
  ORISTUDIO_CP_COMMANDS,
  ORISTUDIO_CP_COMMAND_GROUPS,
  ORISTUDIO_CP_SOURCE_MAP_OPERATION_IDS,
  cpCommandByOperation,
  cpCommandsForGroup,
  cpRailCommands,
} from './oristudioCpCommands';

describe('oristudio CP command registry', () => {
  it('assigns every source-mapped Oriedita operation to one UI command', () => {
    const commandOperations = ORISTUDIO_CP_COMMANDS.map((command) => command.operationId).sort();
    const sourceOperations = [...ORISTUDIO_CP_SOURCE_MAP_OPERATION_IDS].sort();

    expect(commandOperations).toEqual(sourceOperations);
    expect(new Set(commandOperations).size).toBe(commandOperations.length);
  });

  it('keeps command IDs stable and source-map backed', () => {
    expect(cpCommandByOperation('DrawCreaseFree')).toMatchObject({
      id: 'cp.draw-crease-free',
      group: 'draw',
      upstream: 'MouseHandlerDrawCreaseFree',
      placement: 'left-rail',
    });
    expect(cpCommandByOperation('MoveCreasePattern')).toMatchObject({
      id: 'cp.move-crease-pattern',
      uiStatus: 'out-of-scope-ui',
      placement: 'hidden-ui-only',
    });
    expect(cpCommandByOperation('FoldingEstimate')).toMatchObject({
      uiStatus: 'porting',
      group: 'folding',
    });
    expect(cpCommandByOperation('CreaseMakeMountain')).toMatchObject({
      uiStatus: 'ready',
      selectionRequirement: 'selected lines',
    });
    expect(cpCommandByOperation('CreaseMove')).toMatchObject({
      uiStatus: 'ready',
      toolSteps: ['Pick source point', 'Pick destination point'],
    });
    expect(cpCommandByOperation('CreaseMove4p')).toMatchObject({
      uiStatus: 'ready',
      toolSteps: [
        'Pick source first point',
        'Pick source second point',
        'Pick target first point',
        'Pick target second point',
      ],
    });
    expect(cpCommandByOperation('CreaseDeleteIntersecting')).toMatchObject({
      uiStatus: 'ready',
      toolSteps: ['Pick drag start point', 'Pick drag end point'],
    });
    expect(cpCommandByOperation('SelectLineIntersecting')).toMatchObject({
      uiStatus: 'ready',
      toolSteps: ['Pick drag start point', 'Pick drag end point'],
    });
    expect(cpCommandByOperation('FixInaccurate')).toMatchObject({
      uiStatus: 'ready',
      selectionRequirement: 'selected folding lines',
    });
  });

  it('keeps the left rail populated by ordered command groups', () => {
    const groupIds = ORISTUDIO_CP_COMMAND_GROUPS.map((group) => group.id);
    expect(groupIds).toEqual([
      'select-edit',
      'draw',
      'construct',
      'transform',
      'color',
      'annotations',
      'generators',
      'measure',
      'check-fix',
      'folding',
      'file',
    ]);

    for (const group of ORISTUDIO_CP_COMMAND_GROUPS) {
      expect(cpCommandsForGroup(group.id).length, `${group.id} should have commands`).toBeGreaterThan(0);
    }
    expect(cpRailCommands().some((command) => command.group === 'file')).toBe(false);
    expect(cpRailCommands().some((command) => command.group === 'draw')).toBe(true);
  });

  it('makes disabled states explainable instead of hiding missing behavior', () => {
    for (const command of ORISTUDIO_CP_COMMANDS) {
      expect(command.label).not.toEqual('');
      expect(command.icon).not.toEqual('');
      expect(command.tooltip).not.toEqual('');
      expect(command.disabledReason).not.toEqual('');
      if (command.uiStatus === 'not-implemented') {
        expect(command.disabledReason).toContain('Not implemented');
      }
    }
  });
});
