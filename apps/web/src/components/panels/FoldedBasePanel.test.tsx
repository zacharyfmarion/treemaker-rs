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
  it('renders folded paper by default with icon view options', () => {
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
    expect(rendered.querySelector('[aria-label="Wireframe"]')).not.toBeNull();
    expect(rendered.querySelector('[aria-label="Translucent Layers"]')).not.toBeNull();
    expect(rendered.querySelector('[aria-label="Refresh"]')).toBeNull();
    expect(
      rendered.querySelector(
        '.panel-toolbar > .panel-toolbar__group:last-child .folded-base-view-controls'
      )
    ).not.toBeNull();
    expect(
      rendered.querySelector(
        '.panel-toolbar > .panel-toolbar__group:first-child .folded-base-view-controls'
      )
    ).toBeNull();
    expect(
      rendered.querySelector('.folded-base-panel__body .folded-base-view-controls')
    ).toBeNull();
    expect(rendered.textContent).not.toContain('3 vertices | 1 facets');
    expect(rendered.querySelectorAll('.folded-base-facet')).toHaveLength(1);
    expect(rendered.querySelectorAll('.folded-base-outline')).toHaveLength(2);
    expect(rendered.querySelectorAll('.folded-base-crease')).toHaveLength(0);
    expect(rendered.querySelectorAll('.folded-base-vertex')).toHaveLength(0);
    expect(rendered.textContent).not.toContain('Unavailable for imported CP');

    act(() => {
      rendered.querySelector<HTMLButtonElement>('[aria-label="Wireframe"]')?.click();
    });

    expect(rendered.querySelectorAll('.folded-base-crease')).toHaveLength(3);
    expect(rendered.querySelectorAll('.folded-base-vertex')).toHaveLength(3);

    act(() => {
      rendered.querySelector<HTMLButtonElement>('[aria-label="Wireframe"]')?.click();
    });

    expect(rendered.querySelectorAll('.folded-base-crease')).toHaveLength(0);
    expect(rendered.querySelectorAll('.folded-base-vertex')).toHaveLength(0);

    act(() => {
      rendered.querySelector<HTMLButtonElement>('[aria-label="Translucent Layers"]')?.click();
    });

    expect(rendered.querySelectorAll('.folded-base-crease')).toHaveLength(0);
    expect(rendered.querySelectorAll('.folded-base-vertex')).toHaveLength(0);
    expect(rendered.querySelector('.folded-base-canvas')?.getAttribute('data-translucent')).toBe(
      'true'
    );
  });

  it('shows folded-base solve errors in the empty state', () => {
    const rendered = renderPanel({
      fold: {
        file_spec: 1.2,
        frame_classes: ['creasePattern'],
        vertices_coords: [],
        edges_vertices: [],
        faces_vertices: [],
      },
      folded_base: null,
      folded_base_error: 'Layer ordering failed',
    });

    expect(rendered.textContent).toContain('Layer ordering failed');
    expect(rendered.querySelector('[aria-label="Refresh"]')).toBeNull();
  });
});
