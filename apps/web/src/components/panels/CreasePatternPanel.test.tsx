import { renderToStaticMarkup } from 'react-dom/server';
import { beforeEach, describe, expect, it } from 'vitest';
import { DEFAULT_CREASE_COLOR_MODE } from '../../lib/sampleProject';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { CreasePatternPanel } from './CreasePatternPanel';

const initialWorkspaceState = useWorkspaceStore.getInitialState();

describe('CreasePatternPanel', () => {
  beforeEach(() => {
    useWorkspaceStore.setState(initialWorkspaceState, true);
  });

  it('labels crease color controls without abbreviations and defaults to crease roles', () => {
    const markup = renderToStaticMarkup(<CreasePatternPanel />);

    expect(useWorkspaceStore.getState().creaseColorMode).toBe(DEFAULT_CREASE_COLOR_MODE);
    expect(markup).toContain('Color by');
    expect(markup).toContain('Fold types');
    expect(markup).toContain('Crease roles');
    expect(markup).toContain('Color by axial, gusset, ridge, hinge, and pseudohinge roles');
    expect(markup).not.toContain('MVF');
    expect(markup).not.toContain('AGRH');
  });
});
