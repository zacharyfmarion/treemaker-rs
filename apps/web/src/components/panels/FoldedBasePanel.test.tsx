import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, describe, expect, it } from 'vitest';
import type { FoldArtifacts, FoldDocument, FoldedBaseSnapshot } from '../../engine/types';
import { createSampleProject } from '../../lib/sampleProject';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { TooltipProvider } from '../ui/Tooltip';
import { FoldedBasePanel } from './FoldedBasePanel';

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

let root: Root | null = null;
let container: HTMLDivElement | null = null;

function renderPanel(foldArtifacts: FoldArtifacts) {
  useWorkspaceStore.setState(
    {
      ...useWorkspaceStore.getInitialState(),
      project: createSampleProject(),
      documentMode: 'crease-pattern',
      importedCreasePattern: null,
      status: 'crease_pattern_ready',
      engineReady: true,
      foldArtifacts,
    },
    true
  );

  container = document.createElement('div');
  document.body.append(container);
  root = createRoot(container);
  act(() => {
    root?.render(
      <TooltipProvider>
        <FoldedBasePanel />
      </TooltipProvider>
    );
  });
  return container;
}

afterEach(() => {
  if (root) {
    act(() => {
      root?.unmount();
    });
  }
  container?.remove();
  root = null;
  container = null;
  useWorkspaceStore.setState(useWorkspaceStore.getInitialState(), true);
});

describe('FoldedBasePanel', () => {
  it('renders folded-base artifacts for imported crease patterns', () => {
    const fold: FoldDocument = {
      file_spec: 1.2,
      frame_classes: ['creasePattern'],
      vertices_coords: [
        [0, 0],
        [1, 0],
        [1, 1],
      ],
      edges_vertices: [
        [0, 1],
        [1, 2],
        [2, 0],
      ],
      edges_assignment: ['B', 'M', 'B'],
      faces_vertices: [[0, 1, 2]],
    };
    const foldedBase: FoldedBaseSnapshot = {
      vertices: fold.vertices_coords.map((coord, index) => ({
        id: index,
        source_vertex: index,
        loc: { x: coord[0] ?? 0, y: coord[1] ?? 0 },
        paper_loc: { x: coord[0] ?? 0, y: coord[1] ?? 0 },
        depth: 0,
        elevation: 0,
        is_border: true,
      })),
      creases: [
        { id: 0, source_crease: 0, vertices: [0, 1], kind: 0, fold: 3 },
        { id: 1, source_crease: 1, vertices: [1, 2], kind: 0, fold: 1 },
        { id: 2, source_crease: 2, vertices: [2, 0], kind: 0, fold: 3 },
      ],
      facets: [{ id: 0, source_facet: 0, vertices: [0, 1, 2], color: 1, order: 0 }],
    };

    const rendered = renderPanel({ fold, folded_base: foldedBase });

    expect(rendered.querySelector('[aria-label="Folded base"]')).not.toBeNull();
    expect(rendered.textContent).toContain('3 vertices | 1 facets');
    expect(rendered.textContent).not.toContain('Unavailable for imported CP');
  });
});
