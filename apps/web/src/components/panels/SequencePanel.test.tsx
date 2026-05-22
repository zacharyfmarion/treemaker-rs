import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, describe, expect, it } from 'vitest';
import type { FoldArtifacts, FoldDocument, SequencePlan, SequenceStateSnapshot } from '../../engine/types';
import { createSampleProject } from '../../lib/sampleProject';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { SequencePanel } from './SequencePanel';

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

let root: Root | null = null;
let container: HTMLDivElement | null = null;

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

describe('SequencePanel', () => {
  it('renders before and after previews for a simple fold step', () => {
    const rendered = renderPanel(simplePlan());

    expect(rendered.querySelector('[aria-label="Sequence step visual"]')).not.toBeNull();
    expect(rendered.querySelector('[aria-label="Before CP unfolded"]')).not.toBeNull();
    expect(rendered.querySelector('[aria-label="After Folded target"]')).not.toBeNull();
    expect(rendered.querySelectorAll('.sequence-preview-crease--highlight')).toHaveLength(4);
    expect(rendered.textContent).toContain('Make a valley fold on crease 6');
  });

  it('highlights unsupported regions instead of hiding missing transforms', () => {
    const plan = simplePlan();
    plan.status = 'partial';
    plan.steps = [
      {
        kind: 'unsupported_region',
        id: 'step-1',
        label: 'Unsupported collapse region',
        before_state: 'target',
        region: {
          id: 'unresolved-1',
          creases: [1, 2],
          faces: [0],
          reason: 'complex transform not implemented',
        },
      },
    ];

    const rendered = renderPanel(plan);

    expect(rendered.textContent).toContain('Unsupported collapse region');
    expect(rendered.querySelector('[aria-label="Before CP target"]')).not.toBeNull();
    expect(rendered.querySelectorAll('.sequence-preview-crease--highlight')).toHaveLength(4);
    expect(rendered.querySelector('.sequence-preview-face--highlight')).not.toBeNull();
  });
});

function renderPanel(plan: SequencePlan) {
  const fold = plan.states[0]?.document ?? simpleFold();
  useWorkspaceStore.setState(
    {
      ...useWorkspaceStore.getInitialState(),
      project: createSampleProject(),
      documentMode: 'crease-pattern',
      status: 'crease_pattern_ready',
      engineReady: true,
      foldArtifacts: { fold } satisfies FoldArtifacts,
      sequencePlan: plan,
    },
    true
  );

  container = document.createElement('div');
  document.body.append(container);
  root = createRoot(container);
  act(() => {
    root?.render(<SequencePanel />);
  });
  return container;
}

function simplePlan(): SequencePlan {
  const unfolded = sequenceState('unfolded', simpleFold(['B', 'B', 'B', 'B', 'B', 'B', 'F']), []);
  const target = sequenceState('target', simpleFold(), [6]);
  return {
    status: 'complete',
    steps: [
      {
        kind: 'simple_fold',
        id: 'step-1',
        label: 'Make a valley fold on crease 6',
        affected_creases: [6],
        affected_faces: [0, 1],
        before_state: 'unfolded',
        after_state: 'target',
      },
    ],
    states: [unfolded, target],
    diagnostics: [],
    unresolved_regions: [],
    search: {
      states_explored: 2,
      branches_pruned: 0,
      repeated_states: 0,
      timed_out: false,
      budget_exhausted: false,
      best_unresolved_creases: 0,
    },
  };
}

function sequenceState(
  id: string,
  document: FoldDocument,
  activeCreases: number[]
): SequenceStateSnapshot {
  return {
    id,
    document,
    active_creases: activeCreases,
    face_orders: [],
    folded_vertices: document.vertices_coords.map((coord) => [coord[0] ?? 0, coord[1] ?? 0]),
    unresolved_regions: [],
    diagnostics: [],
  };
}

function simpleFold(
  assignments: FoldDocument['edges_assignment'] = ['B', 'B', 'B', 'B', 'B', 'B', 'V']
): FoldDocument {
  return {
    file_spec: 1.2,
    frame_classes: ['creasePattern'],
    vertices_coords: [
      [0, 0],
      [1, 0],
      [1, 1],
      [0, 1],
      [0, 0.5],
      [1, 0.5],
    ],
    edges_vertices: [
      [0, 1],
      [1, 5],
      [5, 2],
      [2, 3],
      [3, 4],
      [4, 0],
      [4, 5],
    ],
    edges_assignment: assignments,
    edges_foldAngle: [null, null, null, null, null, null, assignments?.[6] === 'F' ? 0 : 180],
    faces_vertices: [
      [0, 1, 5, 4],
      [4, 5, 2, 3],
    ],
  };
}
