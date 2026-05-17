import { describe, expect, it } from 'vitest';
import { useWorkspaceStore } from './store';

describe('workspace store slices', () => {
  it('composes project, editing, and crease-pattern state', () => {
    const state = useWorkspaceStore.getState();

    expect(state.project.nodes).toEqual([]);
    expect(state.status).toBe('loading_engine');
    expect(state.selection).toEqual({ kind: 'tree' });
    expect(state.toolMode).toBe('select');
    expect(state.creaseColorMode).toBe('mvf');
    expect(state.currentFileName).toBe('Untitled.tmd5');
    expect(state.createNewProject).toBeTypeOf('function');
    expect(state.openProject).toBeTypeOf('function');
    expect(state.saveProject).toBeTypeOf('function');
    expect(state.addNodeAt).toBeTypeOf('function');
    expect(state.buildCreasePattern).toBeTypeOf('function');
  });
});
