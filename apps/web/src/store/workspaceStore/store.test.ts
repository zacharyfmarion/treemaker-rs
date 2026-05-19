import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type {
  ConditionKind,
  ConditionSnapshot,
  EditReport,
  EdgeSnapshot,
  FoldArtifacts,
  FoldDocument,
  NodeSnapshot,
  OptimizationReport,
  PaperSettings,
  PathSnapshot,
  TreeEdit,
  TreeSnapshot,
  TreeSummary,
} from '../../engine/types';
import { projectFromSnapshot } from '../../engine/snapshotMapper';
import type { FileService, SaveBinaryFileOptions, SaveTextFileOptions } from '../../platform/fileService';
import { DEFAULT_CREASE_COLOR_MODE } from '../../lib/sampleProject';
import { useLayoutStore } from '../layoutStore';

const engineMocks = vi.hoisted(() => ({
  createBlankTree: vi.fn(),
  createStarterTree: vi.fn(),
  ensureTreeHandle: vi.fn(),
  getEngine: vi.fn(),
  initializeBlankTree: vi.fn(),
  loadTreeFromText: vi.fn(),
}));

const exportMocks = vi.hoisted(() => ({
  renderCreasePatternPng: vi.fn(async () => new Uint8Array([1, 2, 3])),
  serializeCreasePatternSvg: vi.fn(() => '<svg role="img"></svg>'),
}));

vi.mock('../../lib/creaseExport', () => exportMocks);

vi.mock('./engineRuntime', async (importOriginal) => {
  const actual = await importOriginal<typeof import('./engineRuntime')>();
  return {
    ...actual,
    createBlankTree: engineMocks.createBlankTree,
    createStarterTree: engineMocks.createStarterTree,
    ensureTreeHandle: engineMocks.ensureTreeHandle,
    getEngine: engineMocks.getEngine,
    initializeBlankTree: engineMocks.initializeBlankTree,
    loadTreeFromText: engineMocks.loadTreeFromText,
  };
});

import type { EngineClient } from './engineRuntime';
import { useWorkspaceStore } from './store';

type SnapshotOptions = Partial<
  Pick<
    TreeSnapshot,
    'nodes' | 'edges' | 'paths' | 'vertices' | 'creases' | 'facets' | 'conditions'
  >
> & {
  paper?: Partial<PaperSettings>;
  summary?: Partial<TreeSummary>;
};

const savedSnapshots = new Map<string, TreeSnapshot>();

const initialWorkspaceState = useWorkspaceStore.getInitialState();
const initialLayoutState = useLayoutStore.getInitialState();

function cloneSnapshot(snapshot: TreeSnapshot): TreeSnapshot {
  return JSON.parse(JSON.stringify(snapshot)) as TreeSnapshot;
}

function nodeSnapshot(
  id: number,
  loc = { x: id / 10, y: id / 10 },
  overrides: Partial<NodeSnapshot> = {}
): NodeSnapshot {
  return {
    id,
    label: `n${id}`,
    loc,
    is_leaf: id !== 1,
    is_pinned: false,
    is_conditioned: false,
    owner: 'Tree',
    ...overrides,
  };
}

function edgeSnapshot(
  id: number,
  nodes: [number, number],
  overrides: Partial<EdgeSnapshot> = {}
): EdgeSnapshot {
  return {
    id,
    label: `e${id}`,
    nodes,
    length: 1,
    strain: 0,
    stiffness: 1,
    is_conditioned: false,
    ...overrides,
  };
}

function pathSnapshot(id: number, nodes: [number, number]): PathSnapshot {
  return {
    id,
    nodes,
    is_leaf: true,
    is_active: true,
    is_feasible: true,
    is_border: false,
    is_polygon: false,
    is_conditioned: false,
  };
}

function nodeFixedCondition(node = 1): ConditionKind {
  return {
    type: 'node_fixed',
    node,
    x_fixed: true,
    y_fixed: false,
    x_fix_value: 0.25,
    y_fix_value: 0,
  };
}

function conditionSnapshot(index: number, kind = nodeFixedCondition()): ConditionSnapshot {
  return {
    index,
    is_feasible: true,
    kind,
  };
}

function makeSnapshot(options: SnapshotOptions = {}): TreeSnapshot {
  const paper: PaperSettings = {
    width: 1,
    height: 1,
    scale: 0.1,
    has_symmetry: false,
    sym_loc: { x: 0.5, y: 0.5 },
    sym_angle: 90,
    ...options.paper,
  };
  const nodes = options.nodes ?? [];
  const edges = options.edges ?? [];
  const paths = options.paths ?? [];
  const vertices = options.vertices ?? [];
  const creases = options.creases ?? [];
  const facets = options.facets ?? [];
  const conditions = options.conditions ?? [];
  const summary: TreeSummary = {
    scale: paper.scale,
    is_feasible: true,
    cp_status: creases.length > 0 ? 'built' : 'ok',
    nodes: nodes.length,
    edges: edges.length,
    paths: paths.length,
    vertices: vertices.length,
    creases: creases.length,
    facets: facets.length,
    leaf_nodes: nodes.filter((node) => node.is_leaf).length,
    conditions: conditions.length,
    conditioned_nodes: nodes.filter((node) => node.is_conditioned).length,
    conditioned_edges: edges.filter((edge) => edge.is_conditioned).length,
    conditioned_paths: paths.filter((path) => path.is_conditioned).length,
    ...options.summary,
  };
  return {
    summary,
    cp_status_report: {
      status: summary.cp_status,
      bad_edges: [],
      bad_polys: [],
      bad_vertices: [],
      bad_creases: [],
      bad_facets: [],
    },
    paper,
    nodes,
    edges,
    paths,
    vertices,
    creases,
    facets,
    conditions,
  };
}

function seedSnapshot(): TreeSnapshot {
  return makeSnapshot({
    nodes: [
      nodeSnapshot(1, { x: 0.5, y: 0.5 }, { label: 'root', is_leaf: false }),
      nodeSnapshot(2, { x: 0.2, y: 0.2 }, { label: 'tip' }),
    ],
    edges: [edgeSnapshot(1, [1, 2])],
    paths: [pathSnapshot(1, [1, 2])],
  });
}

function foldArtifactsFromSnapshot(snapshot: TreeSnapshot): FoldArtifacts {
  if (snapshot.vertices.length === 0 || snapshot.creases.length === 0 || snapshot.facets.length === 0) {
    throw { code: 'invalid_operation', message: 'build a crease pattern before exporting FOLD artifacts' };
  }

  const fold = {
    file_spec: 1.2,
    file_creator: 'store-test',
    frame_title: 'Test crease pattern',
    frame_classes: ['creasePattern'],
    vertices_coords: snapshot.vertices.map((vertex) => [vertex.loc.x, vertex.loc.y]),
    edges_vertices: snapshot.creases.map(
      (crease) => [crease.vertices[0] - 1, crease.vertices[1] - 1] as [number, number]
    ),
    edges_assignment: snapshot.creases.map(() => 'M' as const),
    edges_foldAngle: snapshot.creases.map(() => -180),
    faces_vertices: snapshot.facets.map((facet) => facet.vertices.map((vertex) => vertex - 1)),
  };

  return {
    fold,
    folded_base: {
      vertices: snapshot.vertices.map((vertex) => ({
        id: vertex.id,
        source_vertex: vertex.id,
        loc: vertex.loc,
        paper_loc: vertex.loc,
        depth: 0,
        elevation: 0,
        is_border: false,
      })),
      creases: snapshot.creases.map((crease) => ({
        id: crease.id,
        source_crease: crease.id,
        vertices: [crease.vertices[0], crease.vertices[1]] as [number, number],
        kind: crease.kind,
        fold: crease.fold,
      })),
      facets: snapshot.facets.map((facet) => ({
        id: facet.id,
        source_facet: facet.id,
        vertices: facet.vertices,
        color: facet.color,
        order: 0,
      })),
    },
    simulation_model: {
      fold,
      crease_params: [],
    },
  };
}

function foldArtifactsFromFold(fold: FoldDocument): FoldArtifacts {
  return {
    fold,
    folded_base: {
      vertices: fold.vertices_coords.map((coord, index) => ({
        id: index,
        source_vertex: index,
        loc: { x: coord[0] ?? 0, y: coord[1] ?? 0 },
        paper_loc: { x: coord[0] ?? 0, y: coord[1] ?? 0 },
        depth: 0,
        elevation: 0,
        is_border: fold.edges_vertices.some(
          (edge, edgeIndex) =>
            fold.edges_assignment?.[edgeIndex] === 'B' &&
            (edge[0] === index || edge[1] === index)
        ),
      })),
      creases: fold.edges_vertices.map((vertices, index) => ({
        id: index,
        source_crease: index,
        vertices,
        kind: 0,
        fold: fold.edges_assignment?.[index] === 'M' ? 1 : fold.edges_assignment?.[index] === 'V' ? 2 : 3,
      })),
      facets: fold.faces_vertices.map((vertices, index) => ({
        id: index,
        source_facet: index,
        vertices,
        color: index % 2 === 0 ? 1 : 2,
        order: index,
      })),
    },
    simulation_model:
      fold.faces_vertices.length > 0
        ? {
            fold,
            crease_params: [],
          }
        : null,
    simulation_model_error: fold.faces_vertices.length > 0 ? null : 'Simulation requires faces',
  };
}

function nextId<T extends { id: number }>(items: T[]): number {
  return Math.max(0, ...items.map((item) => item.id)) + 1;
}

function deleteNodeFromSnapshot(snapshot: TreeSnapshot, deletedId: number): TreeSnapshot {
  const nodeMap = new Map<number, number>();
  snapshot.nodes.forEach((node) => {
    if (node.id !== deletedId) nodeMap.set(node.id, nodeMap.size + 1);
  });
  const nodes = snapshot.nodes
    .filter((node) => nodeMap.has(node.id))
    .map((node) => ({ ...node, id: nodeMap.get(node.id)! }));
  const edges = snapshot.edges
    .filter((edge) => edge.nodes.every((node) => nodeMap.has(node)))
    .map((edge, index) => ({
      ...edge,
      id: index + 1,
      nodes: edge.nodes.map((node) => nodeMap.get(node)!) as [number, number],
    }));
  const paths = snapshot.paths
    .filter((path) => path.nodes.every((node) => nodeMap.has(node)))
    .map((path, index) => ({
      ...path,
      id: index + 1,
      nodes: path.nodes.map((node) => nodeMap.get(node)!),
    }));
  return makeSnapshot({ ...snapshot, nodes, edges, paths });
}

function refreshMockTopology(snapshot: TreeSnapshot): TreeSnapshot {
  const degree = new Map(snapshot.nodes.map((node) => [node.id, 0]));
  for (const edge of snapshot.edges) {
    degree.set(edge.nodes[0], (degree.get(edge.nodes[0]) ?? 0) + 1);
    degree.set(edge.nodes[1], (degree.get(edge.nodes[1]) ?? 0) + 1);
  }
  const nodes = snapshot.nodes.map((node) => ({
    ...node,
    is_leaf: snapshot.nodes.length > 1 && (degree.get(node.id) ?? 0) <= 1,
  }));
  const leafIds = new Set(nodes.filter((node) => node.is_leaf).map((node) => node.id));
  const conditions = snapshot.conditions
    .filter((condition) => {
      switch (condition.kind.type) {
        case 'node_symmetric':
          return leafIds.has(condition.kind.node);
        case 'nodes_paired':
          return leafIds.has(condition.kind.node1) && leafIds.has(condition.kind.node2);
        default:
          return true;
      }
    })
    .map((condition, index) => ({ ...condition, index: index + 1 }));
  return makeSnapshot({ ...snapshot, nodes, conditions });
}

function createMockEngineApi(initialSnapshot: TreeSnapshot) {
  let snapshotState = cloneSnapshot(initialSnapshot);
  let saveCount = 0;
  let nextConditionId = Math.max(0, ...snapshotState.conditions.map((condition) => condition.index)) + 1;

  const setSnapshot = (snapshot: TreeSnapshot) => {
    snapshotState = cloneSnapshot(snapshot);
    return cloneSnapshot(snapshotState);
  };

  const api = {
    replaceSnapshot: setSnapshot,
    get snapshotState() {
      return cloneSnapshot(snapshotState);
    },
    newDesign: vi.fn(async () => 1),
    loadTmd: vi.fn(async () => 1),
    freeTree: vi.fn(async () => undefined),
    snapshot: vi.fn(async () => cloneSnapshot(snapshotState)),
    saveTmd5: vi.fn(async () => {
      const text = `saved-${++saveCount}`;
      savedSnapshots.set(text, cloneSnapshot(snapshotState));
      return text;
    }),
    exportV4: vi.fn(async () => 'exported-v4'),
    exportFold: vi.fn(async () => JSON.stringify(foldArtifactsFromSnapshot(snapshotState).fold)),
    foldArtifacts: vi.fn(async () => foldArtifactsFromSnapshot(snapshotState)),
    flatFoldArtifacts: vi.fn(async (foldJson: string) =>
      foldArtifactsFromFold(JSON.parse(foldJson) as FoldDocument)
    ),
    optimizeScale: vi.fn(async (): Promise<OptimizationReport> => {
      const oldScale = snapshotState.paper.scale;
      snapshotState = makeSnapshot({
        ...snapshotState,
        paper: { ...snapshotState.paper, scale: oldScale + 0.05 },
        summary: { ...snapshotState.summary, is_feasible: true },
      });
      return {
        kind: 'scale',
        converged: true,
        old_scale: oldScale,
        new_scale: snapshotState.paper.scale,
        is_feasible: true,
        message: 'scale optimized',
      };
    }),
    optimizeEdges: vi.fn(async (): Promise<OptimizationReport> => ({
      kind: 'edges',
      converged: true,
      old_scale: snapshotState.paper.scale,
      new_scale: snapshotState.paper.scale,
      is_feasible: true,
      message: 'edges optimized',
    })),
    optimizeStrain: vi.fn(async (): Promise<OptimizationReport> => ({
      kind: 'strain',
      converged: true,
      old_scale: snapshotState.paper.scale,
      new_scale: snapshotState.paper.scale,
      is_feasible: true,
      message: 'strain optimized',
    })),
    buildCreasePattern: vi.fn(async () => {
      snapshotState = makeSnapshot({
        ...snapshotState,
        vertices: [
          { id: 1, loc: { x: 0, y: 0 } },
          { id: 2, loc: { x: 1, y: 0 } },
          { id: 3, loc: { x: 1, y: 1 } },
        ],
        creases: [{ id: 1, kind: 0, vertices: [1, 2], fold: 3 }],
        facets: [{ id: 1, vertices: [1, 2, 3], color: 1 }],
      });
      return cloneSnapshot(snapshotState);
    }),
    applyEdit: vi.fn(async (_handle: number, edit: TreeEdit): Promise<EditReport> => {
      let createdNode: number | undefined;
      let createdEdge: number | undefined;

      switch (edit.type) {
        case 'add_node': {
          createdNode = nextId(snapshotState.nodes);
          const nodes = [
            ...snapshotState.nodes,
            nodeSnapshot(createdNode, edit.loc, { label: edit.label ?? `n${createdNode}` }),
          ];
          const edges = [...snapshotState.edges];
          if (edit.connect_to !== undefined) {
            createdEdge = nextId(edges);
            edges.push(
              edgeSnapshot(createdEdge, [edit.connect_to, createdNode], {
                length: edit.edge_length ?? 1,
              })
            );
          }
          snapshotState = refreshMockTopology(makeSnapshot({ ...snapshotState, nodes, edges }));
          break;
        }
        case 'move_node':
          snapshotState = makeSnapshot({
            ...snapshotState,
            nodes: snapshotState.nodes.map((node) =>
              node.id === edit.id ? { ...node, loc: edit.loc } : node
            ),
          });
          break;
        case 'delete_node':
          snapshotState = refreshMockTopology(deleteNodeFromSnapshot(snapshotState, edit.id));
          break;
        case 'update_node_label':
          snapshotState = makeSnapshot({
            ...snapshotState,
            nodes: snapshotState.nodes.map((node) =>
              node.id === edit.id ? { ...node, label: edit.label } : node
            ),
          });
          break;
        case 'add_edge':
          createdEdge = nextId(snapshotState.edges);
          snapshotState = refreshMockTopology(makeSnapshot({
            ...snapshotState,
            edges: [
              ...snapshotState.edges,
              edgeSnapshot(createdEdge, [edit.node1, edit.node2], {
                label: edit.label ?? `e${createdEdge}`,
                length: edit.length ?? 1,
              }),
            ],
          }));
          break;
        case 'delete_edge':
          snapshotState = refreshMockTopology(makeSnapshot({
            ...snapshotState,
            edges: snapshotState.edges.filter((edge) => edge.id !== edit.id),
          }));
          break;
        case 'update_edge':
          snapshotState = makeSnapshot({
            ...snapshotState,
            edges: snapshotState.edges.map((edge) =>
              edge.id === edit.id
                ? {
                    ...edge,
                    label: edit.label ?? edge.label,
                    length: edit.length ?? edge.length,
                    strain: edit.strain ?? edge.strain,
                    stiffness: edit.stiffness ?? edge.stiffness,
                  }
                : edge
            ),
          });
          break;
        case 'update_paper':
          snapshotState = makeSnapshot({
            ...snapshotState,
            paper: {
              ...snapshotState.paper,
              width: edit.width,
              height: edit.height,
              scale: edit.scale ?? snapshotState.paper.scale,
            },
          });
          break;
        case 'set_symmetry':
          snapshotState = makeSnapshot({
            ...snapshotState,
            paper: {
              ...snapshotState.paper,
              has_symmetry: edit.has_symmetry,
              sym_loc: edit.sym_loc ?? snapshotState.paper.sym_loc,
              sym_angle: edit.sym_angle ?? snapshotState.paper.sym_angle,
            },
          });
          break;
        case 'add_condition':
          snapshotState = makeSnapshot({
            ...snapshotState,
            conditions: [
              ...snapshotState.conditions,
              { index: nextConditionId++, is_feasible: true, kind: edit.kind },
            ],
          });
          break;
        case 'update_condition':
          snapshotState = makeSnapshot({
            ...snapshotState,
            conditions: snapshotState.conditions.map((condition) =>
              condition.index === edit.id ? { ...condition, kind: edit.kind } : condition
            ),
          });
          break;
        case 'delete_condition':
          snapshotState = makeSnapshot({
            ...snapshotState,
            conditions: snapshotState.conditions.filter((condition) => condition.index !== edit.id),
          });
          break;
      }

      return {
        snapshot: cloneSnapshot(snapshotState),
        created_node: createdNode,
        created_edge: createdEdge,
      };
    }),
  };

  return api;
}

type TestEngineApi = ReturnType<typeof createMockEngineApi>;

function configureEngine(api: TestEngineApi) {
  const engine = api as unknown as EngineClient;
  engineMocks.getEngine.mockReset().mockResolvedValue(engine);
  engineMocks.ensureTreeHandle.mockReset().mockResolvedValue({ api: engine, treeHandle: 1 });
  engineMocks.initializeBlankTree.mockReset().mockImplementation(async () => api.snapshot());
  engineMocks.createBlankTree.mockReset().mockImplementation(async () => {
    const snapshot = makeSnapshot();
    api.replaceSnapshot(snapshot);
    return cloneSnapshot(snapshot);
  });
  engineMocks.createStarterTree.mockReset().mockImplementation(async () => {
    const snapshot = seedSnapshot();
    api.replaceSnapshot(snapshot);
    return cloneSnapshot(snapshot);
  });
  engineMocks.loadTreeFromText.mockReset().mockImplementation(async (_engine: EngineClient, text: string) => {
    const snapshot = savedSnapshots.get(text) ?? api.snapshotState;
    api.replaceSnapshot(snapshot);
    return cloneSnapshot(snapshot);
  });
}

function loadSnapshotIntoStore(snapshot: TreeSnapshot, title = 'Seed project') {
  useWorkspaceStore.setState({
    project: projectFromSnapshot(snapshot, title),
    documentMode: 'tree',
    importedCreasePattern: null,
    projectLoadId: useWorkspaceStore.getState().projectLoadId + 1,
    currentFileName: 'seed.tmd5',
    currentFilePath: null,
    projectMessage: null,
    status: snapshot.creases.length > 0 ? 'crease_pattern_ready' : 'optimized',
    dirty: false,
    engineReady: true,
    error: null,
    lastOptimization: null,
    designViewportFitRequestId: 0,
    historyPast: [],
    historyFuture: [],
    historyBusy: false,
    selection: { kind: 'tree' },
    toolMode: 'select',
    symmetryAuthoringPairs: [],
    clipboard: null,
    clipboardPasteCount: 0,
    creaseColorMode: DEFAULT_CREASE_COLOR_MODE,
    foldArtifacts: null,
    foldArtifactError: null,
  });
}

function resetStores(snapshot = makeSnapshot()) {
  localStorage.clear();
  savedSnapshots.clear();
  useWorkspaceStore.setState(initialWorkspaceState, true);
  useLayoutStore.setState(initialLayoutState, true);
  const api = createMockEngineApi(snapshot);
  configureEngine(api);
  vi.spyOn(window, 'confirm').mockReturnValue(true);
  return api;
}

function createFileService(
  file: { text: string; name: string; path: string | null } | null = null
): FileService & {
  openTextFile: ReturnType<typeof vi.fn>;
  saveTextFile: ReturnType<typeof vi.fn>;
  saveBinaryFile: ReturnType<typeof vi.fn>;
} {
  return {
    surface: 'web',
    supportsNativeDialogs: false,
    openTextFile: vi.fn(async () => file),
    saveTextFile: vi.fn(async (options: SaveTextFileOptions) => ({
      name: options.suggestedName,
      path: options.path ?? `/tmp/${options.suggestedName}`,
    })),
    saveBinaryFile: vi.fn(async (options: SaveBinaryFileOptions) => ({
      name: options.suggestedName,
      path: null,
    })),
  };
}

async function flushAsyncWork() {
  await Promise.resolve();
  await Promise.resolve();
}

describe('workspace store slices', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    resetStores();
  });

  afterEach(async () => {
    await flushAsyncWork();
  });

  it('composes project, history, editing, clipboard, conditions, and crease-pattern state', () => {
    const state = useWorkspaceStore.getState();

    expect(state.project.nodes).toEqual([]);
    expect(state.documentMode).toBe('tree');
    expect(state.importedCreasePattern).toBeNull();
    expect(state.status).toBe('loading_engine');
    expect(state.selection).toEqual({ kind: 'tree' });
    expect(state.toolMode).toBe('select');
    expect(state.symmetryAuthoringPairs).toEqual([]);
    expect(state.creaseColorMode).toBe(DEFAULT_CREASE_COLOR_MODE);
    expect(state.foldArtifacts).toBeNull();
    expect(state.designViewportFitRequestId).toBe(0);
    expect(state.historyPast).toEqual([]);
    expect(state.clipboard).toBeNull();
    expect(state.projectLoadId).toBe(0);
    expect(state.currentFileName).toBe('Untitled.tmd5');
    expect(state.createNewProject).toBeTypeOf('function');
    expect(state.openProject).toBeTypeOf('function');
    expect(state.loadCreasePatternText).toBeTypeOf('function');
    expect(state.saveProject).toBeTypeOf('function');
    expect(state.exportFold).toBeTypeOf('function');
    expect(state.undo).toBeTypeOf('function');
    expect(state.copySelection).toBeTypeOf('function');
    expect(state.updatePaper).toBeTypeOf('function');
    expect(state.addCondition).toBeTypeOf('function');
    expect(state.addNodeAt).toBeTypeOf('function');
    expect(state.addNodeWithSymmetry).toBeTypeOf('function');
    expect(state.previewSymmetryLeafPairs).toBeTypeOf('function');
    expect(state.optimizeEdges).toBeTypeOf('function');
    expect(state.buildCreasePattern).toBeTypeOf('function');
  });

  it('initializes projects, loads text, saves, exports, and maintains recents', async () => {
    const api = resetStores(seedSnapshot());
    const fileService = createFileService({
      text: 'opened text',
      name: 'opened.tmd5',
      path: '/tmp/opened.tmd5',
    });

    await useWorkspaceStore.getState().initEngine();
    expect(useWorkspaceStore.getState().engineReady).toBe(true);
    expect(useWorkspaceStore.getState().project.nodes).toHaveLength(2);
    const initializedLoadId = useWorkspaceStore.getState().projectLoadId;

    await useWorkspaceStore.getState().loadProjectText('loaded text', {
      title: 'Loaded design',
      filename: 'loaded.tmd5',
      path: '/tmp/loaded.tmd5',
    });
    expect(useWorkspaceStore.getState().projectLoadId).toBe(initializedLoadId + 1);
    expect(useWorkspaceStore.getState()).toMatchObject({
      currentFileName: 'loaded.tmd5',
      currentFilePath: '/tmp/loaded.tmd5',
      dirty: false,
      projectMessage: 'Loaded loaded.tmd5',
    });
    expect(useWorkspaceStore.getState().recentProjects[0]).toMatchObject({
      id: '/tmp/loaded.tmd5',
      title: 'Loaded design',
      filename: 'loaded.tmd5',
      text: 'loaded text',
    });

    await expect(useWorkspaceStore.getState().openProject(fileService)).resolves.toBe(true);
    expect(fileService.openTextFile).toHaveBeenCalledWith({
      title: 'Open Ori Studio Project or Crease Pattern',
      extensions: ['tmd', 'tmd4', 'tmd5', 'fold', 'cp'],
    });

    await expect(useWorkspaceStore.getState().saveProject(fileService)).resolves.toBe(true);
    expect(fileService.saveTextFile).toHaveBeenCalledWith(
      expect.objectContaining({
        title: 'Save Ori Studio Project',
        path: '/tmp/opened.tmd5',
        extensions: ['tmd5'],
      })
    );
    expect(useWorkspaceStore.getState().dirty).toBe(false);

    await expect(useWorkspaceStore.getState().saveProjectAs(fileService)).resolves.toBe(true);
    expect(fileService.saveTextFile).toHaveBeenLastCalledWith(
      expect.objectContaining({
        title: 'Save Ori Studio Project As',
        path: null,
      })
    );

    await expect(useWorkspaceStore.getState().exportV4(fileService)).resolves.toBe(true);
    expect(api.exportV4).toHaveBeenCalledWith(1);

    await useWorkspaceStore.getState().buildCreasePattern();
    await expect(useWorkspaceStore.getState().exportFold(fileService)).resolves.toBe(true);
    expect(api.exportFold).toHaveBeenCalledWith(1);
    expect(fileService.saveTextFile).toHaveBeenLastCalledWith(
      expect.objectContaining({ title: 'Export FOLD Document', extensions: ['fold'] })
    );

    await expect(useWorkspaceStore.getState().exportSvg(fileService)).resolves.toBe(true);
    expect(exportMocks.serializeCreasePatternSvg).toHaveBeenCalledWith(
      useWorkspaceStore.getState().project,
      DEFAULT_CREASE_COLOR_MODE
    );

    await expect(useWorkspaceStore.getState().exportPng(fileService)).resolves.toBe(true);
    expect(exportMocks.renderCreasePatternPng).toHaveBeenCalledWith(
      useWorkspaceStore.getState().project,
      DEFAULT_CREASE_COLOR_MODE
    );
    expect(fileService.saveBinaryFile).toHaveBeenCalledWith(
      expect.objectContaining({ extensions: ['png'], mimeType: 'image/png' })
    );

    useWorkspaceStore.setState({ dirty: true });
    await useWorkspaceStore.getState().autosaveProject();
    expect(useWorkspaceStore.getState().recentProjects[0]).toMatchObject({
      id: 'treemaker.autosave.v1',
      filename: useWorkspaceStore.getState().currentFileName,
    });

    useWorkspaceStore.getState().clearProjectMessage();
    expect(useWorkspaceStore.getState().projectMessage).toBeNull();
  });

  it('loads CP-only documents and gates tree-only persistence', async () => {
    const api = resetStores(seedSnapshot());
    loadSnapshotIntoStore(seedSnapshot());
    const activatePanel = vi.fn();
    useLayoutStore.setState({ activatePanel });
    const fileService = createFileService({
      text: [
        '1 0 0 1 0',
        '1 1 0 1 1',
        '1 1 1 0 1',
        '1 0 1 0 0',
        '2 0 0 1 1',
      ].join('\n'),
      name: 'square.cp',
      path: '/tmp/square.cp',
    });

    await expect(useWorkspaceStore.getState().openProject(fileService)).resolves.toBe(true);

    expect(useWorkspaceStore.getState()).toMatchObject({
      documentMode: 'crease-pattern',
      currentFileName: 'square.cp',
      currentFilePath: '/tmp/square.cp',
      dirty: false,
      status: 'crease_pattern_ready',
    });
    expect(useWorkspaceStore.getState().importedCreasePattern?.source.format).toBe('cp');
    expect(useWorkspaceStore.getState().project.creases.length).toBeGreaterThan(0);
    expect(useWorkspaceStore.getState().project.facets.length).toBeGreaterThan(0);
    expect(useWorkspaceStore.getState().foldArtifacts?.folded_base?.facets.length).toBeGreaterThan(
      0
    );
    expect(api.flatFoldArtifacts).toHaveBeenCalledOnce();
    expect(activatePanel).toHaveBeenCalledWith('crease-pattern');

    await expect(useWorkspaceStore.getState().saveProject(fileService)).resolves.toBe(false);
    expect(useWorkspaceStore.getState().error?.message).toBe(
      'Imported crease patterns are exported, not saved as Ori Studio projects'
    );

    await expect(useWorkspaceStore.getState().exportFold(fileService)).resolves.toBe(true);
    expect(fileService.saveTextFile).toHaveBeenLastCalledWith(
      expect.objectContaining({ title: 'Export FOLD Document', extensions: ['fold'] })
    );
  });

  it('applies editing and condition actions through the engine', async () => {
    const api = resetStores(
      makeSnapshot({
        ...seedSnapshot(),
        conditions: [conditionSnapshot(1)],
      })
    );
    loadSnapshotIntoStore(api.snapshotState);

    await useWorkspaceStore.getState().addNodeAt({ x: 0.75, y: 0.75 }, 1);
    expect(useWorkspaceStore.getState().project.nodes.map((node) => node.id)).toEqual([1, 2, 3]);
    expect(useWorkspaceStore.getState().selection).toEqual({ kind: 'node', id: 3 });
    expect(useWorkspaceStore.getState().status).toBe('needs_optimization');
    expect(useWorkspaceStore.getState().historyPast.at(-1)?.label).toBe('Add node');

    await useWorkspaceStore.getState().moveNode(3, { x: 0.8, y: 0.7 });
    expect(useWorkspaceStore.getState().project.nodes.find((node) => node.id === 3)?.loc).toEqual({
      x: 0.8,
      y: 0.7,
    });

    await useWorkspaceStore.getState().updateNodeLabel(3, 'new tip');
    expect(useWorkspaceStore.getState().project.nodes.find((node) => node.id === 3)?.label).toBe(
      'new tip'
    );

    await useWorkspaceStore.getState().addEdge(2, 3);
    expect(useWorkspaceStore.getState().selection).toEqual({ kind: 'edge', id: 3 });

    await useWorkspaceStore
      .getState()
      .updateEdge(3, { label: 'span', length: 2, strain: 0.1, stiffness: 4 });
    expect(useWorkspaceStore.getState().project.edges.find((edge) => edge.id === 3)).toMatchObject({
      label: 'span',
      length: 2,
      strain: 0.1,
      stiffness: 4,
    });

    useWorkspaceStore.getState().select({ kind: 'multi', nodes: [1, 2], edges: [], paths: [], creases: [], facets: [], conditions: [] });
    useWorkspaceStore.getState().selectPathBetweenSelectedNodes();
    expect(useWorkspaceStore.getState().selection).toEqual({ kind: 'path', id: 1 });

    useWorkspaceStore.getState().selectAll();
    expect(useWorkspaceStore.getState().selection).toMatchObject({ kind: 'multi', nodes: [1, 2, 3] });
    useWorkspaceStore.getState().selectNone();
    expect(useWorkspaceStore.getState().selection).toEqual({ kind: 'tree' });
    useWorkspaceStore.getState().setToolMode('node');
    expect(useWorkspaceStore.getState().toolMode).toBe('node');

    await useWorkspaceStore.getState().updatePaper({ width: 2, height: 3 });
    expect(useWorkspaceStore.getState().project.paper).toMatchObject({ width: 2, height: 3 });

    await useWorkspaceStore
      .getState()
      .setSymmetry({ hasSymmetry: true, symLoc: { x: 0.25, y: 0.75 }, symAngle: 45 });
    expect(useWorkspaceStore.getState().project).toMatchObject({
      hasSymmetry: true,
      paper: { symLoc: { x: 0.25, y: 0.75 }, symAngle: 45 },
    });

    await useWorkspaceStore.getState().addCondition(nodeFixedCondition(2));
    expect(useWorkspaceStore.getState().project.conditions).toHaveLength(2);
    await useWorkspaceStore.getState().deleteCondition(1);
    expect(useWorkspaceStore.getState().project.conditions.map((condition) => condition.id)).toEqual([2]);
    await useWorkspaceStore.getState().clearConditions();
    expect(useWorkspaceStore.getState().project.conditions).toEqual([]);

    useWorkspaceStore.getState().selectAll();
    await useWorkspaceStore.getState().deleteSelection();
    expect(useWorkspaceStore.getState().project.nodes).toEqual([]);
    expect(useWorkspaceStore.getState().projectMessage).toBe('Cleared design');
  });

  it('creates mirrored branches from an axial parent in one history entry', async () => {
    const api = resetStores(
      makeSnapshot({
        paper: { has_symmetry: true },
        nodes: [nodeSnapshot(1, { x: 0.5, y: 0.5 }, { label: 'axis', is_leaf: false })],
      })
    );
    loadSnapshotIntoStore(api.snapshotState);

    await useWorkspaceStore.getState().addNodeWithSymmetry({ x: 0.25, y: 0.72 }, 1);

    expect(useWorkspaceStore.getState().project.nodes.map((node) => node.loc)).toEqual([
      { x: 0.5, y: 0.5 },
      { x: 0.25, y: 0.72 },
      { x: 0.75, y: 0.72 },
    ]);
    expect(useWorkspaceStore.getState().project.edges.map((edge) => edge.nodes)).toEqual([
      [1, 2],
      [1, 3],
    ]);
    expect(useWorkspaceStore.getState().project.conditions.map((condition) => condition.kind)).toEqual([
      { type: 'nodes_paired', node1: 2, node2: 3 },
    ]);
    expect(useWorkspaceStore.getState().selection).toMatchObject({ kind: 'multi', nodes: [2, 3] });
    expect(useWorkspaceStore.getState().historyPast).toHaveLength(1);
    expect(useWorkspaceStore.getState().historyPast[0].label).toBe('Add mirrored branch');
  });

  it('keeps internal mirror links after branching from a mirrored leaf', async () => {
    const api = resetStores(
      makeSnapshot({
        paper: { has_symmetry: true },
        nodes: [
          nodeSnapshot(1, { x: 0.5, y: 0.5 }, { label: 'axis', is_leaf: false }),
          nodeSnapshot(2, { x: 0.28, y: 0.5 }),
          nodeSnapshot(3, { x: 0.72, y: 0.5 }),
        ],
        edges: [edgeSnapshot(1, [1, 2]), edgeSnapshot(2, [1, 3])],
        conditions: [
          conditionSnapshot(1, { type: 'nodes_paired', node1: 2, node2: 3 }),
        ],
      })
    );
    loadSnapshotIntoStore(api.snapshotState);

    await useWorkspaceStore.getState().addNodeWithSymmetry({ x: 0.16, y: 0.7 }, 2);

    expect(useWorkspaceStore.getState().project.conditions.map((condition) => condition.kind)).toEqual([
      { type: 'nodes_paired', node1: 4, node2: 5 },
    ]);
    expect(useWorkspaceStore.getState().symmetryAuthoringPairs).toEqual([
      { node1: 2, node2: 3 },
      { node1: 4, node2: 5 },
    ]);

    await useWorkspaceStore.getState().addNodeWithSymmetry({ x: 0.14, y: 0.3 }, 2);

    const nodeLocs = useWorkspaceStore.getState().project.nodes.map((node) => node.loc);
    expect(nodeLocs).toHaveLength(7);
    expect(nodeLocs[0]).toEqual({ x: 0.5, y: 0.5 });
    expect(nodeLocs[1]).toEqual({ x: 0.28, y: 0.5 });
    expect(nodeLocs[2]).toEqual({ x: 0.72, y: 0.5 });
    expect(nodeLocs[3]).toEqual({ x: 0.16, y: 0.7 });
    expect(nodeLocs[4]).toEqual({ x: 0.84, y: 0.7 });
    expect(nodeLocs[5]).toEqual({ x: 0.14, y: 0.3 });
    expect(nodeLocs[6]?.x).toBeCloseTo(0.86);
    expect(nodeLocs[6]?.y).toBeCloseTo(0.3);
    expect(useWorkspaceStore.getState().project.conditions.map((condition) => condition.kind)).toEqual([
      { type: 'nodes_paired', node1: 4, node2: 5 },
      { type: 'nodes_paired', node1: 6, node2: 7 },
    ]);
  });

  it('draws an axial segment once in symmetry mode', async () => {
    const api = resetStores(
      makeSnapshot({
        paper: { has_symmetry: true },
        nodes: [nodeSnapshot(1, { x: 0.5, y: 0.5 }, { label: 'axis', is_leaf: false })],
      })
    );
    loadSnapshotIntoStore(api.snapshotState);

    await useWorkspaceStore.getState().addNodeWithSymmetry({ x: 0.506, y: 0.72 }, 1);

    expect(useWorkspaceStore.getState().project.nodes).toHaveLength(2);
    expect(useWorkspaceStore.getState().project.nodes[1].loc.x).toBeCloseTo(0.5);
    expect(useWorkspaceStore.getState().project.nodes[1].loc.y).toBeCloseTo(0.72);
    expect(useWorkspaceStore.getState().project.edges.map((edge) => edge.nodes)).toEqual([[1, 2]]);
    expect(useWorkspaceStore.getState().project.conditions).toEqual([]);
    expect(useWorkspaceStore.getState().projectMessage).toBe('Added axial node');
  });

  it('applies symmetry leaf pairs and skips duplicate conditions', async () => {
    const api = resetStores(
      makeSnapshot({
        paper: { has_symmetry: true },
        nodes: [
          nodeSnapshot(1, { x: 0.5, y: 0.5 }, { label: 'root', is_leaf: false }),
          nodeSnapshot(2, { x: 0.2, y: 0.25 }),
          nodeSnapshot(3, { x: 0.8, y: 0.25 }),
          nodeSnapshot(4, { x: 0.25, y: 0.75 }),
          nodeSnapshot(5, { x: 0.75, y: 0.75 }),
          nodeSnapshot(6, { x: 0.504, y: 0.9 }),
        ],
        conditions: [
          conditionSnapshot(1, { type: 'nodes_paired', node1: 2, node2: 3 }),
        ],
      })
    );
    loadSnapshotIntoStore(api.snapshotState);

    const preview = useWorkspaceStore.getState().previewSymmetryLeafPairs();
    expect(preview.pairs).toEqual([{ node1: 4, node2: 5, distance: 0 }]);
    expect(preview.onAxis.map((item) => item.node)).toEqual([6]);

    await useWorkspaceStore.getState().applySymmetryLeafPairs();

    expect(useWorkspaceStore.getState().project.conditions.map((condition) => condition.kind)).toEqual([
      { type: 'nodes_paired', node1: 2, node2: 3 },
      { type: 'nodes_paired', node1: 4, node2: 5 },
      { type: 'node_symmetric', node: 6 },
    ]);
    expect(useWorkspaceStore.getState().historyPast.at(-1)?.label).toBe('Apply symmetry pairs');
  });

  it('moves paired leaf nodes together in one history entry', async () => {
    const api = resetStores(
      makeSnapshot({
        paper: { has_symmetry: true },
        nodes: [
          nodeSnapshot(1, { x: 0.5, y: 0.5 }, { label: 'root', is_leaf: false }),
          nodeSnapshot(2, { x: 0.2, y: 0.25 }),
          nodeSnapshot(3, { x: 0.8, y: 0.25 }),
        ],
        conditions: [
          conditionSnapshot(1, { type: 'nodes_paired', node1: 2, node2: 3 }),
        ],
      })
    );
    loadSnapshotIntoStore(api.snapshotState);

    await useWorkspaceStore.getState().moveNodeWithSymmetry(2, { x: 0.3, y: 0.4 });

    expect(useWorkspaceStore.getState().project.nodes.find((node) => node.id === 2)?.loc).toEqual({
      x: 0.3,
      y: 0.4,
    });
    expect(useWorkspaceStore.getState().project.nodes.find((node) => node.id === 3)?.loc).toEqual({
      x: 0.7,
      y: 0.4,
    });
    expect(useWorkspaceStore.getState().historyPast).toHaveLength(1);
    expect(useWorkspaceStore.getState().historyPast[0].label).toBe('Move mirrored nodes');
  });

  it('updates mirrored flap edge lengths together', async () => {
    const api = resetStores(
      makeSnapshot({
        paper: { has_symmetry: true },
        nodes: [
          nodeSnapshot(1, { x: 0.5, y: 0.5 }, { label: 'root', is_leaf: false }),
          nodeSnapshot(2, { x: 0.2, y: 0.3 }),
          nodeSnapshot(3, { x: 0.8, y: 0.3 }),
        ],
        edges: [edgeSnapshot(1, [1, 2]), edgeSnapshot(2, [1, 3])],
        conditions: [
          conditionSnapshot(1, { type: 'nodes_paired', node1: 2, node2: 3 }),
        ],
      })
    );
    loadSnapshotIntoStore(api.snapshotState);

    await useWorkspaceStore.getState().updateEdge(1, { length: 2.5 });

    expect(useWorkspaceStore.getState().project.edges.map((edge) => edge.length)).toEqual([
      2.5,
      2.5,
    ]);
    expect(useWorkspaceStore.getState().historyPast).toHaveLength(1);
    expect(useWorkspaceStore.getState().historyPast[0].label).toBe('Edit mirrored edges');
  });

  it('deletes a selected design node from the canonical engine snapshot', async () => {
    const api = resetStores(
      makeSnapshot({
        nodes: [
          nodeSnapshot(1, { x: 0.5, y: 0.5 }, { label: 'root', is_leaf: false }),
          nodeSnapshot(2, { x: 0.2, y: 0.2 }, { label: 'left' }),
          nodeSnapshot(3, { x: 0.8, y: 0.2 }, { label: 'right' }),
        ],
        edges: [edgeSnapshot(1, [1, 2]), edgeSnapshot(2, [1, 3])],
        paths: [pathSnapshot(1, [1, 2]), pathSnapshot(2, [1, 3]), pathSnapshot(3, [2, 3])],
      })
    );
    loadSnapshotIntoStore(api.snapshotState);

    useWorkspaceStore.getState().select({ kind: 'node', id: 2 });
    await useWorkspaceStore.getState().deleteSelection();

    expect(api.applyEdit).toHaveBeenCalledWith(1, { type: 'delete_node', id: 2 });
    expect(useWorkspaceStore.getState().project.nodes.map((node) => [node.id, node.label])).toEqual([
      [1, 'root'],
      [2, 'right'],
    ]);
    expect(useWorkspaceStore.getState().project.edges.map((edge) => [edge.id, edge.nodes])).toEqual([
      [1, [1, 2]],
    ]);
    expect(useWorkspaceStore.getState()).toMatchObject({
      selection: { kind: 'tree' },
      status: 'needs_optimization',
      error: null,
      dirty: true,
    });
  });

  it('copies, cuts, and pastes selected topology', async () => {
    resetStores(seedSnapshot());
    loadSnapshotIntoStore(seedSnapshot());

    useWorkspaceStore.getState().select({
      kind: 'multi',
      nodes: [1, 2],
      edges: [],
      paths: [],
      creases: [],
      facets: [],
      conditions: [],
    });
    useWorkspaceStore.getState().copySelection();

    expect(useWorkspaceStore.getState().clipboard).toMatchObject({
      nodes: [
        { sourceId: 1, label: 'root' },
        { sourceId: 2, label: 'tip' },
      ],
      edges: [{ sourceId: 1, sourceNodes: [1, 2] }],
    });

    await useWorkspaceStore.getState().pasteClipboard();
    expect(useWorkspaceStore.getState().project.nodes.map((node) => node.id)).toEqual([1, 2, 3, 4]);
    expect(useWorkspaceStore.getState().selection).toMatchObject({
      kind: 'multi',
      nodes: [3, 4],
    });
    expect(useWorkspaceStore.getState().clipboardPasteCount).toBe(1);

    await useWorkspaceStore.getState().cutSelection();
    expect(useWorkspaceStore.getState().clipboard?.nodes.map((node) => node.sourceId)).toEqual([3, 4]);
    expect(useWorkspaceStore.getState().project.nodes.map((node) => node.id)).toEqual([1, 2]);
  });

  it('records checkpoints and restores snapshots through undo and redo', async () => {
    resetStores(seedSnapshot());
    loadSnapshotIntoStore(seedSnapshot());

    await useWorkspaceStore.getState().addNodeAt({ x: 0.8, y: 0.8 }, 1);
    expect(useWorkspaceStore.getState().project.nodes).toHaveLength(3);
    expect(useWorkspaceStore.getState().historyPast).toHaveLength(1);

    await useWorkspaceStore.getState().undo();
    expect(useWorkspaceStore.getState().project.nodes).toHaveLength(2);
    expect(useWorkspaceStore.getState().historyFuture).toHaveLength(1);
    expect(useWorkspaceStore.getState().projectMessage).toBe('Undid Add node');

    await useWorkspaceStore.getState().redo();
    expect(useWorkspaceStore.getState().project.nodes).toHaveLength(3);
    expect(useWorkspaceStore.getState().historyPast).toHaveLength(1);
    expect(useWorkspaceStore.getState().projectMessage).toBe('Redid Add node');

    useWorkspaceStore.getState().clearHistory();
    expect(useWorkspaceStore.getState().historyPast).toEqual([]);
    expect(useWorkspaceStore.getState().historyFuture).toEqual([]);
  });

  it('optimizes, builds crease patterns, toggles color mode, and foregrounds the CP pane', async () => {
    const api = resetStores(seedSnapshot());
    loadSnapshotIntoStore(seedSnapshot());
    const activatePanel = vi.fn();
    useLayoutStore.setState({ activatePanel });

    const initialFitRequestId = useWorkspaceStore.getState().designViewportFitRequestId;
    await useWorkspaceStore.getState().optimizeScale();
    expect(useWorkspaceStore.getState().status).toBe('optimized');
    expect(useWorkspaceStore.getState().lastOptimization).toMatchObject({ kind: 'scale' });
    expect(useWorkspaceStore.getState().designViewportFitRequestId).toBe(
      initialFitRequestId + 1
    );

    await useWorkspaceStore.getState().optimizeEdges();
    expect(useWorkspaceStore.getState().lastOptimization).toMatchObject({ kind: 'edges' });
    expect(useWorkspaceStore.getState().designViewportFitRequestId).toBe(
      initialFitRequestId + 1
    );

    await useWorkspaceStore.getState().optimizeStrain();
    expect(useWorkspaceStore.getState().lastOptimization).toMatchObject({ kind: 'strain' });
    expect(useWorkspaceStore.getState().designViewportFitRequestId).toBe(
      initialFitRequestId + 1
    );

    await useWorkspaceStore.getState().buildCreasePattern();
    expect(useWorkspaceStore.getState().status).toBe('crease_pattern_ready');
    expect(useWorkspaceStore.getState().project.creases).toHaveLength(1);
    expect(useWorkspaceStore.getState().foldArtifacts?.fold.vertices_coords).toHaveLength(3);
    expect(useWorkspaceStore.getState().refreshFoldArtifacts).toBeTypeOf('function');
    expect(api.foldArtifacts).toHaveBeenCalledWith(1);
    expect(activatePanel).toHaveBeenCalledWith('crease-pattern');

    useWorkspaceStore.getState().setCreaseColorMode('mvf');
    expect(useWorkspaceStore.getState().creaseColorMode).toBe('mvf');
  });

  it('does not mark CP ready when build returns no drawable crease pattern', async () => {
    const api = resetStores(seedSnapshot());
    loadSnapshotIntoStore(seedSnapshot());
    api.buildCreasePattern.mockResolvedValueOnce(seedSnapshot());

    await useWorkspaceStore.getState().buildCreasePattern();

    expect(useWorkspaceStore.getState().status).toBe('optimized');
    expect(useWorkspaceStore.getState().project.creases).toHaveLength(0);
    expect(useWorkspaceStore.getState().project.facets).toHaveLength(0);
    expect(useWorkspaceStore.getState().error).toEqual({
      code: 'invalid_operation',
      message: 'Build CP completed but did not produce drawable crease-pattern geometry.',
    });
    expect(api.foldArtifacts).not.toHaveBeenCalled();
  });

  it('blocks building a crease pattern before optimization succeeds', async () => {
    const api = resetStores(seedSnapshot());
    loadSnapshotIntoStore(seedSnapshot());
    useWorkspaceStore.setState({ status: 'needs_optimization', error: null });

    await useWorkspaceStore.getState().buildCreasePattern();

    expect(api.buildCreasePattern).not.toHaveBeenCalled();
    expect(useWorkspaceStore.getState().status).toBe('needs_optimization');
    expect(useWorkspaceStore.getState().error).toEqual({
      code: 'invalid_operation',
      message: 'Optimize Scale before building the crease pattern',
    });
  });

  it('surfaces engine errors on mutating actions', async () => {
    const api = resetStores(seedSnapshot());
    loadSnapshotIntoStore(seedSnapshot());
    api.applyEdit.mockRejectedValueOnce({ code: 'invalid_operation', message: 'nope' });

    await useWorkspaceStore.getState().addNodeAt({ x: 0.4, y: 0.4 });

    expect(useWorkspaceStore.getState().status).toBe('error');
    expect(useWorkspaceStore.getState().error).toEqual({
      code: 'invalid_operation',
      message: 'nope',
    });
  });
});
