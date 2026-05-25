import { describe, expect, it } from 'vitest';
import { cpRailActions } from './oristudioCpActions';
import {
  instructionsForCpAction,
  instructionsForOrieditaAction,
} from './oristudioCpToolInstructions';

describe('Oriedita CP tool instructions', () => {
  it('resolves Rabbit Ear to the detailed Oriedita instruction text', () => {
    expect(instructionsForOrieditaAction('rabbitEarAction')).toMatchObject({
      intro: ['Draw lines toward inner center.'],
      steps: [
        'Select point A.',
        'Select point B.',
        'Select point C.',
        'Three lines toward inner center are drawn.',
      ],
    });
  });

  it('resolves rail aliases by their upstream Oriedita action', () => {
    const railActions = cpRailActions();

    expect(
      instructionsForCpAction(
        railActions.find(
          (action) => action.kind === 'command' && action.operationId === 'DrawCreaseAngleRestricted5'
        )
      )?.intro?.[0]
    ).toBe('Draw angle restricted line with selected angle restriction.');

    expect(
      instructionsForCpAction(
        railActions.find(
          (action) => action.kind === 'command' && action.operationId === 'AngleSystem'
        )
      )?.steps
    ).toEqual(['Select 2 points to set angle offset.', 'Select a target line to extend to.']);

    expect(
      instructionsForCpAction(
        railActions.find(
          (action) => action.kind === 'command' && action.operationId === 'DrawCreaseAngleRestricted'
        )
      )?.intro?.[0]
    ).toBe('Draw converging lines with angle offset.');

    expect(
      instructionsForCpAction(
        railActions.find(
          (action) => action.kind === 'command' && action.operationId === 'FoldableLineInput'
        )
      )?.notes
    ).toEqual(['Select a line to extend it.']);
  });

  it('returns null for unmapped Oriedita actions', () => {
    expect(instructionsForOrieditaAction('missingAction')).toBeNull();
    expect(instructionsForCpAction(undefined)).toBeNull();
  });
});
