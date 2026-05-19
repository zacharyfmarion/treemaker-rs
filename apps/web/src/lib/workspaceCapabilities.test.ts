import { describe, expect, it } from 'vitest';
import type { AppStatus, DocumentMode, Selection } from './sampleProject';
import { getNextDocumentAction, getWorkspaceCapabilities } from './workspaceCapabilities';

const treeSelection: Selection = { kind: 'tree' };

function capabilities({
  documentMode = 'tree',
  status = 'ready',
  edgeCount = 0,
  creaseCount = 0,
  facetCount = 0,
  engineReady = true,
  hasImportedCreasePattern = false,
}: {
  documentMode?: DocumentMode;
  status?: AppStatus;
  edgeCount?: number;
  creaseCount?: number;
  facetCount?: number;
  engineReady?: boolean;
  hasImportedCreasePattern?: boolean;
} = {}) {
  return getWorkspaceCapabilities({
    documentMode,
    engineReady,
    status,
    edgeCount,
    creaseCount,
    facetCount,
    hasImportedCreasePattern,
    hasSimulationModel: false,
    historyPastCount: 0,
    historyFutureCount: 0,
    clipboard: null,
    selection: treeSelection,
  });
}

describe('workspace capabilities', () => {
  it('disables optimize and build when tree documents have no edges', () => {
    const state = capabilities();

    expect(state['optimize.scale'].enabled).toBe(false);
    expect(state['optimize.scale'].reason).toBe('Add at least one tree edge before optimizing');
    expect(state['cp.build'].enabled).toBe(false);
    expect(state['cp.build'].reason).toBe('Add tree edges, then optimize before building the crease pattern');
    expect(getNextDocumentAction(state)).toBe('optimize.scale');
  });

  it('enables optimization before CP build and build after optimization', () => {
    const needsOptimization = capabilities({ status: 'needs_optimization', edgeCount: 2 });
    expect(needsOptimization['optimize.scale'].enabled).toBe(true);
    expect(needsOptimization['cp.build'].enabled).toBe(false);

    const optimized = capabilities({ status: 'optimized', edgeCount: 2 });
    expect(optimized['optimize.scale'].enabled).toBe(true);
    expect(optimized['cp.build'].enabled).toBe(true);
    expect(optimized['cp.build'].label).toBe('Build CP');
    expect(getNextDocumentAction(optimized)).toBe('cp.build');
  });

  it('allows rebuilding an existing generated crease pattern', () => {
    const state = capabilities({ status: 'crease_pattern_ready', edgeCount: 2, creaseCount: 4, facetCount: 1 });

    expect(state['cp.build'].enabled).toBe(true);
    expect(state['cp.build'].label).toBe('Rebuild CP');
    expect(state['file.exportFold'].enabled).toBe(true);
  });

  it('disables tree-only commands for imported crease-pattern documents', () => {
    const state = capabilities({
      documentMode: 'crease-pattern',
      status: 'crease_pattern_ready',
      creaseCount: 5,
      facetCount: 1,
      hasImportedCreasePattern: true,
    });

    expect(state['optimize.scale'].enabled).toBe(false);
    expect(state['optimize.scale'].reason).toBe('Optimization requires a TreeMaker tree document');
    expect(state['cp.build'].enabled).toBe(false);
    expect(state['cp.build'].reason).toBe('Build CP requires a TreeMaker tree document');
    expect(state['file.save'].enabled).toBe(false);
    expect(state['file.exportFold'].enabled).toBe(true);
    expect(state['file.exportSvg'].enabled).toBe(true);
    expect(state['view.foldedBase'].enabled).toBe(true);
    expect(state['foldedBase.refresh'].enabled).toBe(false);
    expect(getNextDocumentAction(state)).toBe(null);
  });

  it('disables workflow actions while the engine is busy or unavailable', () => {
    for (const status of ['loading_engine', 'optimizing', 'building_crease_pattern'] as const) {
      const state = capabilities({ status, edgeCount: 2, engineReady: status !== 'loading_engine' });

      expect(state['optimize.scale'].enabled).toBe(false);
      expect(state['cp.build'].enabled).toBe(false);
    }

    const errorState = capabilities({ status: 'error', edgeCount: 2 });
    expect(errorState['optimize.scale'].enabled).toBe(false);
    expect(errorState['cp.build'].enabled).toBe(false);
  });
});
