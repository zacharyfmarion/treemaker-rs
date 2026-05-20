import { describe, expect, it, vi } from 'vitest';
import { handleAppKeyDown } from './appKeyboard';
import { createSampleProject, type Selection } from './sampleProject';
import { selectEverything } from './selection';

function createActions(selection: Selection) {
  return {
    deleteSelection: vi.fn(),
    getSelection: vi.fn(() => selection),
    handleMenuAction: vi.fn(),
    selectNone: vi.fn(),
  };
}

describe('app keyboard shortcuts', () => {
  it('clears the active selection on Escape', () => {
    const actions = createActions(selectEverything(createSampleProject()));
    const event = new KeyboardEvent('keydown', { key: 'Escape', cancelable: true });

    expect(handleAppKeyDown(event, actions)).toBe(true);

    expect(event.defaultPrevented).toBe(true);
    expect(actions.selectNone).toHaveBeenCalledOnce();
  });

  it('ignores Escape when nothing is selected', () => {
    const actions = createActions({ kind: 'tree' });
    const event = new KeyboardEvent('keydown', { key: 'Escape', cancelable: true });

    expect(handleAppKeyDown(event, actions)).toBe(false);

    expect(event.defaultPrevented).toBe(false);
    expect(actions.selectNone).not.toHaveBeenCalled();
  });

  it('keeps text input Escape available to the focused control', () => {
    const actions = createActions(selectEverything(createSampleProject()));
    const input = document.createElement('input');
    const event = new KeyboardEvent('keydown', {
      key: 'Escape',
      bubbles: true,
      cancelable: true,
    });
    let handled = true;
    input.addEventListener('keydown', (keyboardEvent) => {
      handled = handleAppKeyDown(keyboardEvent, actions);
    });

    input.dispatchEvent(event);

    expect(handled).toBe(false);
    expect(event.defaultPrevented).toBe(false);
    expect(actions.selectNone).not.toHaveBeenCalled();
  });

  it('preserves Select All routing through the shared command layer', () => {
    const actions = createActions({ kind: 'tree' });
    const event = new KeyboardEvent('keydown', {
      key: 'a',
      metaKey: true,
      cancelable: true,
    });

    expect(handleAppKeyDown(event, actions)).toBe(true);

    expect(event.defaultPrevented).toBe(true);
    expect(actions.handleMenuAction).toHaveBeenCalledWith('edit.selectAll');
  });
});
