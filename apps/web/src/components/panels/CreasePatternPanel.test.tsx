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

  it('labels crease color controls without abbreviations and defaults to the M/V assignment', () => {
    const markup = renderToStaticMarkup(<CreasePatternPanel />);

    expect(useWorkspaceStore.getState().creaseColorMode).toBe(DEFAULT_CREASE_COLOR_MODE);
    expect(markup).toContain('Color by');
    expect(markup).toContain('Crease roles');
    expect(markup).toContain('M/V assignment');
    expect(markup).toContain('Color by mountain, valley, flat, and border folds');
    expect(markup).not.toContain('MVF');
    expect(markup).not.toContain('AGRH');
  });
});
