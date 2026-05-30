import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  handleShortcutRuntimeKeyDown,
  registerCpActionShortcutExecutor,
  registerViewportShortcutExecutor,
  shortcutScopeStackForContext,
} from './shortcutRuntime';

const cleanupFns: Array<() => void> = [];

function cleanupWith(dispose: () => void): void {
  cleanupFns.push(dispose);
}

afterEach(() => {
  for (const cleanup of cleanupFns.splice(0)) cleanup();
});

describe('shortcut runtime', () => {
  it('computes scopes from active app surface instead of DOM focus', () => {
    expect(
      shortcutScopeStackForContext({
        documentMode: 'crease-pattern',
        activeEditingSurface: 'crease-pattern',
      })
    ).toEqual(['viewport', 'crease-pattern', 'global']);

    expect(
      shortcutScopeStackForContext({
        documentMode: 'tree',
        activeEditingSurface: 'tree',
      })
    ).toEqual(['viewport', 'global']);
  });

  it('lets viewport ownership differ from editing ownership', () => {
    const designViewport = vi.fn();
    const cpViewport = vi.fn();
    const menu = vi.fn();
    cleanupWith(registerViewportShortcutExecutor('tree', designViewport));
    cleanupWith(registerViewportShortcutExecutor('crease-pattern', cpViewport));

    const event = new KeyboardEvent('keydown', {
      key: '=',
      metaKey: true,
      bubbles: true,
      cancelable: true,
    });

    expect(
      handleShortcutRuntimeKeyDown(event, {
        context: {
          documentMode: 'tree',
          activeEditingSurface: 'tree',
          activeViewportSurface: 'crease-pattern',
        },
        menu,
      })
    ).toBe(true);

    expect(cpViewport).toHaveBeenCalledWith('viewport.zoomIn');
    expect(designViewport).not.toHaveBeenCalled();
  });

  it('runs scoped CP shortcuts before global shortcuts when CP is active', () => {
    const cpAction = vi.fn();
    const menu = vi.fn();
    cleanupWith(registerCpActionShortcutExecutor(cpAction));

    const event = new KeyboardEvent('keydown', {
      key: 'b',
      ctrlKey: true,
      bubbles: true,
      cancelable: true,
    });

    expect(
      handleShortcutRuntimeKeyDown(event, {
        context: {
          documentMode: 'crease-pattern',
          activeEditingSurface: 'crease-pattern',
        },
        menu,
      })
    ).toBe(true);

    expect(cpAction).toHaveBeenCalledWith('cp.action.inward');
    expect(menu).not.toHaveBeenCalled();
  });

  it('routes viewport shortcuts to the active surface executor', () => {
    const designViewport = vi.fn();
    const cpViewport = vi.fn();
    const menu = vi.fn();
    cleanupWith(registerViewportShortcutExecutor('tree', designViewport));
    cleanupWith(registerViewportShortcutExecutor('crease-pattern', cpViewport));

    const event = new KeyboardEvent('keydown', {
      key: '=',
      metaKey: true,
      bubbles: true,
      cancelable: true,
    });

    expect(
      handleShortcutRuntimeKeyDown(event, {
        context: {
          documentMode: 'tree',
          activeEditingSurface: 'tree',
        },
        menu,
      })
    ).toBe(true);

    expect(designViewport).toHaveBeenCalledWith('viewport.zoomIn');
    expect(cpViewport).not.toHaveBeenCalled();
    expect(menu).not.toHaveBeenCalled();
  });

  it('keeps global aliases available through the central runtime', () => {
    const menu = vi.fn();
    const event = new KeyboardEvent('keydown', {
      key: 'Backspace',
      bubbles: true,
      cancelable: true,
    });

    expect(
      handleShortcutRuntimeKeyDown(event, {
        context: {
          documentMode: 'crease-pattern',
          activeEditingSurface: 'crease-pattern',
        },
        menu,
      })
    ).toBe(true);

    expect(menu).toHaveBeenCalledWith('edit.delete');
    expect(event.defaultPrevented).toBe(true);
  });
});
