import { describe, expect, it } from 'vitest';
import {
  cpCommandByOperation,
  type OristudioCpCommandDefinition,
} from './oristudioCpCommands';
import {
  cancelOristudioCpToolState,
  IDLE_ORISTUDIO_CP_TOOL_STATE,
  transitionOristudioCpToolState,
} from './oristudioCpToolState';

function command(operationId: OristudioCpCommandDefinition['operationId']) {
  const definition = cpCommandByOperation(operationId);
  if (!definition) throw new Error(`Missing command ${operationId}`);
  return definition;
}

function ready(operationId: OristudioCpCommandDefinition['operationId']) {
  return {
    ...command(operationId),
    uiStatus: 'ready',
    disabledReason: '',
  } satisfies OristudioCpCommandDefinition;
}

describe('oristudio CP tool state', () => {
  it('selects unavailable commands as blocked but keeps the active command visible', () => {
    const state = transitionOristudioCpToolState(IDLE_ORISTUDIO_CP_TOOL_STATE, {
      type: 'selectCommand',
      command: command('DisplayLengthBetweenPoints1'),
      editable: true,
    });

    expect(state).toMatchObject({
      activeOperationId: 'DisplayLengthBetweenPoints1',
      phase: 'blocked',
      prompt: 'Measure length 1: Not implemented',
      status: 'not-implemented',
      stepIndex: 0,
    });
    expect(state.steps).toEqual(['Pick first point', 'Pick second point']);
  });

  it('starts ready multi-step commands and advances prompts without changing tools', () => {
    const first = transitionOristudioCpToolState(IDLE_ORISTUDIO_CP_TOOL_STATE, {
      type: 'selectCommand',
      command: ready('DrawCreaseFree'),
      editable: true,
    });
    const second = transitionOristudioCpToolState(first, { type: 'advanceStep' });
    const beyondLast = transitionOristudioCpToolState(second, { type: 'advanceStep' });

    expect(first.prompt).toBe('Draw crease: Pick start point');
    expect(second).toMatchObject({
      activeOperationId: 'DrawCreaseFree',
      phase: 'active',
      prompt: 'Draw crease: Pick end point',
      stepIndex: 1,
    });
    expect(beyondLast).toBe(second);
  });

  it('resets step state when switching command modes', () => {
    const drawing = transitionOristudioCpToolState(IDLE_ORISTUDIO_CP_TOOL_STATE, {
      type: 'selectCommand',
      command: ready('DrawCreaseFree'),
      editable: true,
    });
    const nextDrawingStep = transitionOristudioCpToolState(drawing, { type: 'advanceStep' });
    const moving = transitionOristudioCpToolState(nextDrawingStep, {
      type: 'selectCommand',
      command: ready('CreaseMove'),
      editable: true,
    });

    expect(moving).toMatchObject({
      activeOperationId: 'CreaseMove',
      phase: 'active',
      prompt: 'Move selected creases: Pick source point',
      stepIndex: 0,
    });
  });

  it('surfaces command errors against the active tool', () => {
    const active = transitionOristudioCpToolState(IDLE_ORISTUDIO_CP_TOOL_STATE, {
      type: 'selectCommand',
      command: ready('DrawPoint'),
      editable: true,
    });

    expect(
      transitionOristudioCpToolState(active, {
        type: 'commandError',
        message: 'candidate vanished',
      })
    ).toMatchObject({
      activeOperationId: 'DrawPoint',
      phase: 'error',
      prompt: 'Draw point: candidate vanished',
      status: 'error',
    });
  });

  it('cancels active or blocked tools before falling through to selection clearing', () => {
    expect(cancelOristudioCpToolState(IDLE_ORISTUDIO_CP_TOOL_STATE)).toEqual({
      handled: false,
      state: IDLE_ORISTUDIO_CP_TOOL_STATE,
    });

    const blocked = transitionOristudioCpToolState(IDLE_ORISTUDIO_CP_TOOL_STATE, {
      type: 'selectCommand',
      command: command('Fold'),
      editable: true,
    });

    expect(cancelOristudioCpToolState(blocked)).toEqual({
      handled: true,
      state: IDLE_ORISTUDIO_CP_TOOL_STATE,
    });
  });
});
