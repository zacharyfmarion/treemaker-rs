import type { OristudioCpLineColor } from '../engine/oristudioCpTypes';
import {
  ORISTUDIO_CP_COMMAND_GROUPS,
  ORISTUDIO_CP_COMMANDS,
  cpCommandByOperation,
  type OristudioCpCommandDefinition,
  type OristudioCpCommandGroupId,
  type OristudioCpCommandPlacement,
  type OristudioCpCommandUiStatus,
  type OristudioCpOperationId,
} from './oristudioCpCommands';

export type OristudioCpActionKind = 'line-type' | 'command';
export type OristudioCpActionId = `cp.action.${string}`;
export type OristudioCpActionGroupId = 'line-type' | 'edit' | OristudioCpCommandGroupId;
export type OristudioCpLineInputMode = 'fold-line' | 'aux-line';
export type OristudioCpActionInputMode = 'point-sequence' | 'drag-path' | 'drag-line' | 'drag-box';
export const DEFAULT_ORISTUDIO_CP_ACTION_ID =
  'cp.action.crease-select' as const satisfies OristudioCpActionId;

export interface OristudioCpActionGroupDefinition {
  id: OristudioCpActionGroupId;
  label: string;
  railLabel: string;
  order: number;
}

export interface OristudioCpBaseActionDefinition {
  id: OristudioCpActionId;
  kind: OristudioCpActionKind;
  label: string;
  railLabel?: string;
  group: OristudioCpActionGroupId;
  placement: OristudioCpCommandPlacement;
  icon: string;
  railOrder?: number;
  upstreamAction: string;
  upstreamMouseMode?: string;
  tooltip: string;
  uiStatus: OristudioCpCommandUiStatus;
  disabledReason: string;
  shortcut?: string;
}

export interface OristudioCpLineTypeActionDefinition
  extends OristudioCpBaseActionDefinition {
  kind: 'line-type';
  group: 'line-type';
  lineColor: OristudioCpLineColor;
  lineInputMode: OristudioCpLineInputMode;
}

export interface OristudioCpCommandActionDefinition
  extends OristudioCpBaseActionDefinition {
  kind: 'command';
  command: OristudioCpCommandDefinition;
  operationId: OristudioCpOperationId;
  toolSteps?: readonly string[];
  inputMode?: OristudioCpActionInputMode;
  lineInputMode?: OristudioCpLineInputMode;
  repeatable?: boolean;
}

export type OristudioCpActionDefinition =
  | OristudioCpLineTypeActionDefinition
  | OristudioCpCommandActionDefinition;

export const ORISTUDIO_CP_LINE_TYPE_ACTIONS = [
  lineTypeAction('mountain', 'Mountain', 'M', 'Red1', 'colRedAction', 'line-type-mountain'),
  lineTypeAction('valley', 'Valley', 'V', 'Blue2', 'colBlueAction', 'line-type-valley'),
  lineTypeAction('edge', 'Edge', 'E', 'Black0', 'colBlackAction', 'line-type-edge'),
  lineTypeAction('auxiliary', 'Auxiliary', 'A', 'Cyan3', 'colCyanAction', 'line-type-auxiliary'),
] as const satisfies readonly OristudioCpLineTypeActionDefinition[];

const AUXILIARY_DRAW_COMMAND = cpCommandByOperation('DrawCreaseFree');

const ORISTUDIO_CP_EXTRA_ACTION_GROUPS = [
  { id: 'line-type', label: 'Line type', railLabel: 'Type', order: 5 },
  { id: 'edit', label: 'Edit', railLabel: 'Edit', order: 25 },
] as const satisfies readonly OristudioCpActionGroupDefinition[];

export const ORISTUDIO_CP_ACTION_GROUPS: OristudioCpActionGroupDefinition[] = [
  ...ORISTUDIO_CP_EXTRA_ACTION_GROUPS,
  ...ORISTUDIO_CP_COMMAND_GROUPS.map((group) => ({
    id: group.id,
    label: group.label,
    railLabel: group.railLabel,
    order: group.order,
  })),
].sort((a, b) => a.order - b.order);

const ORIEDITA_RAIL_ACTION_OVERRIDES: Partial<
  Record<OristudioCpOperationId, Partial<OristudioCpCommandActionDefinition>>
> = {
  CreaseSelect: {
    label: 'Box Select',
    upstreamAction: 'selectAction',
    upstreamMouseMode: 'CREASE_SELECT_19',
    railOrder: 10,
  },
  SelectLineIntersecting: {
    label: 'Select Overlapping Lines',
    upstreamAction: 'select_lXAction',
    upstreamMouseMode: 'SELECT_LINE_INTERSECTING_68',
    railOrder: 20,
  },
  SelectPolygon: {
    label: 'Polygon Select',
    upstreamAction: 'select_polygonAction',
    upstreamMouseMode: 'SELECT_POLYGON_66',
    railOrder: 30,
  },
  SelectLasso: {
    label: 'Lasso Select',
    upstreamAction: 'selectLassoAction',
    upstreamMouseMode: 'SELECT_LASSO_74',
    railOrder: 40,
  },
  CreaseUnselect: {
    label: 'Box Deselect',
    upstreamAction: 'unselectAction',
    upstreamMouseMode: 'CREASE_UNSELECT_20',
    railOrder: 50,
  },
  UnselectLineIntersecting: {
    label: 'Deselect Overlapping Lines',
    upstreamAction: 'unselect_lXAction',
    upstreamMouseMode: 'UNSELECT_LINE_INTERSECTING_69',
    railOrder: 60,
  },
  UnselectPolygon: {
    label: 'Polygon Deselect',
    upstreamAction: 'unselect_polygonAction',
    upstreamMouseMode: 'UNSELECT_POLYGON_67',
    railOrder: 70,
  },
  UnselectLasso: {
    label: 'Lasso Deselect',
    upstreamAction: 'unselectLassoAction',
    upstreamMouseMode: 'UNSELECT_LASSO_75',
    railOrder: 80,
  },
  DrawCreaseFree: {
    id: 'cp.action.draw-crease',
    label: 'Line',
    group: 'draw',
    upstreamAction: 'drawCreaseFreeAction',
    upstreamMouseMode: 'DRAW_CREASE_FREE_1',
    railOrder: 10,
  },
  DrawCreaseRestricted: {
    label: 'Grid Restricted Line',
    group: 'draw',
    upstreamAction: 'drawCreaseRestrictedAction',
    upstreamMouseMode: 'DRAW_CREASE_RESTRICTED_11',
    railOrder: 20,
  },
  Inward: {
    label: 'Rabbit Ear',
    group: 'draw',
    upstreamAction: 'rabbitEarAction',
    upstreamMouseMode: 'INWARD_8',
    railOrder: 30,
  },
  VertexMakeAngularlyFlatFoldable: {
    label: 'Flat Foldable Line',
    group: 'draw',
    upstreamAction: 'makeFlatFoldableAction',
    upstreamMouseMode: 'VERTEX_MAKE_ANGULARLY_FLAT_FOLDABLE_38',
    railOrder: 40,
  },
  LengthenCrease: {
    label: 'Extend Line',
    group: 'draw',
    upstreamAction: 'lengthenCreaseAction',
    upstreamMouseMode: 'LENGTHEN_CREASE_5',
    railOrder: 50,
  },
  PerpendicularDraw: {
    label: 'Perpendicular Line',
    group: 'draw',
    upstreamAction: 'perpendicularDrawAction',
    upstreamMouseMode: 'PERPENDICULAR_DRAW_9',
    railOrder: 60,
  },
  DrawCreaseAngleRestricted5: {
    label: 'Angle Restricted Line',
    group: 'draw',
    upstreamAction: 'deg2Action',
    upstreamMouseMode: 'DRAW_CREASE_ANGLE_RESTRICTED_5_37',
    railOrder: 70,
  },
  AngleSystem: {
    label: 'Offset Restricted Line',
    group: 'draw',
    upstreamAction: 'deg3Action',
    upstreamMouseMode: 'ANGLE_SYSTEM_16',
    railOrder: 80,
  },
  DrawCreaseAngleRestricted: {
    label: 'Converging Lines',
    group: 'draw',
    upstreamAction: 'deg1Action',
    upstreamMouseMode: 'DRAW_CREASE_ANGLE_RESTRICTED_13',
    railOrder: 90,
  },
  FoldableLineDraw: {
    label: 'Flat Foldable Line (free)',
    group: 'draw',
    upstreamAction: 'foldableLineDrawAction',
    upstreamMouseMode: 'FOLDABLE_LINE_DRAW_71',
    railOrder: 100,
  },
  FoldableLineInput: {
    label: 'Flat Foldable Line (extend)',
    group: 'draw',
    upstreamAction: 'foldableLinePlusGridInputAction',
    upstreamMouseMode: 'FOLDABLE_LINE_INPUT_39',
    railOrder: 110,
  },
  ParallelDraw: {
    label: 'Parallel Line',
    group: 'draw',
    upstreamAction: 'parallelDrawAction',
    upstreamMouseMode: 'PARALLEL_DRAW_40',
    railOrder: 120,
  },
  SymmetricDraw: {
    label: 'Mirror Line',
    group: 'draw',
    upstreamAction: 'symmetricDrawAction',
    upstreamMouseMode: 'SYMMETRIC_DRAW_10',
    railOrder: 130,
  },
  SquareBisector: {
    label: 'Angle Bisector',
    group: 'draw',
    upstreamAction: 'angleBisectorAction',
    upstreamMouseMode: 'SQUARE_BISECTOR_7',
    railOrder: 140,
  },
  FishBoneDraw: {
    label: 'Parallel Alternating Lines',
    group: 'draw',
    upstreamAction: 'fishBoneDrawAction',
    upstreamMouseMode: 'FISH_BONE_DRAW_33',
    railOrder: 150,
  },
  DoubleSymmetricDraw: {
    label: 'Reflect Over Line',
    group: 'draw',
    upstreamAction: 'doubleSymmetricDrawAction',
    upstreamMouseMode: 'DOUBLE_SYMMETRIC_DRAW_35',
    railOrder: 160,
  },
  ContinuousSymmetricDraw: {
    label: 'Reflect Through Lines',
    group: 'draw',
    upstreamAction: 'continuousSymmetricDrawAction',
    upstreamMouseMode: 'CONTINUOUS_SYMMETRIC_DRAW_52',
    railOrder: 170,
  },
  LineSegmentDivision: {
    label: 'Equally Divided Line',
    group: 'draw',
    upstreamAction: 'senbun_b_nyuryokuAction',
    upstreamMouseMode: 'LINE_SEGMENT_DIVISION_27',
    railOrder: 180,
  },
  LineSegmentRatioSet: {
    label: 'Divided Line (ratio)',
    group: 'draw',
    upstreamAction: 'drawLineSegmentInternalDivisionRatioAction',
    upstreamMouseMode: 'LINE_SEGMENT_RATIO_SET_28',
    railOrder: 190,
  },
  PolygonSetNoCorners: {
    label: 'Regular Polygon',
    group: 'draw',
    upstreamAction: 'regularPolygonAction',
    upstreamMouseMode: 'POLYGON_SET_NO_CORNERS_29',
    railOrder: 200,
  },
  VoronoiCreate: {
    group: 'draw',
    upstreamAction: 'voronoiAction',
    upstreamMouseMode: 'VORONOI_CREATE_62',
    railOrder: 210,
  },
  Axiom5: {
    group: 'draw',
    upstreamAction: 'axiom5Action',
    upstreamMouseMode: 'AXIOM_5',
    railOrder: 220,
  },
  Axiom7: {
    group: 'draw',
    upstreamAction: 'axiom7Action',
    upstreamMouseMode: 'AXIOM_7',
    railOrder: 230,
  },
  DeletePoint: {
    label: 'Delete Point',
    group: 'edit',
    upstreamAction: 'vertexDeleteAction',
    upstreamMouseMode: 'DELETE_POINT_15',
    railOrder: 10,
  },
  VertexDeleteOnCrease: {
    label: 'Delete any Vertex',
    group: 'edit',
    upstreamAction: 'v_del_ccAction',
    upstreamMouseMode: 'VERTEX_DELETE_ON_CREASE_41',
    railOrder: 20,
  },
  CreaseDeleteOverlapping: {
    label: 'Delete Coincident Lines',
    group: 'edit',
    upstreamAction: 'del_lAction',
    upstreamMouseMode: 'CREASE_DELETE_OVERLAPPING_64',
    railOrder: 30,
  },
  CreaseDeleteIntersecting: {
    label: 'Delete Overlapping Lines',
    group: 'edit',
    upstreamAction: 'del_l_XAction',
    upstreamMouseMode: 'CREASE_DELETE_INTERSECTING_65',
    railOrder: 40,
  },
};

export const ORISTUDIO_CP_ACTIONS: OristudioCpActionDefinition[] = [
  ...ORISTUDIO_CP_LINE_TYPE_ACTIONS,
  ...ORISTUDIO_CP_COMMANDS.map((command) =>
    commandAction(command, ORIEDITA_RAIL_ACTION_OVERRIDES[command.operationId])
  ),
  ...(AUXILIARY_DRAW_COMMAND
    ? [
        {
          ...commandAction(AUXILIARY_DRAW_COMMAND, {
            id: 'cp.action.draw-auxiliary-line',
            label: 'Draw auxiliary line',
            icon: 'scan-line',
            upstreamAction: 'h_senbun_nyuryokuAction',
            upstreamMouseMode: 'DRAW_CREASE_FREE_1',
            tooltip: 'Draw an Oriedita auxiliary line',
            uiStatus: 'not-implemented',
            disabledReason: 'Auxiliary-line insertion mode is not implemented in the CP kernel yet',
            lineInputMode: 'aux-line',
          }),
          placement: 'left-rail-overflow',
        } satisfies OristudioCpCommandActionDefinition,
      ]
    : []),
];

function lineTypeAction(
  id: string,
  label: string,
  railLabel: string,
  lineColor: OristudioCpLineColor,
  upstreamAction: string,
  icon: string
): OristudioCpLineTypeActionDefinition {
  return {
    id: `cp.action.line-type.${id}`,
    kind: 'line-type',
    label,
    railLabel,
    group: 'line-type',
    placement: 'bottom-toolbar',
    icon,
    upstreamAction,
    tooltip: `Set active line type to ${label.toLowerCase()}`,
    uiStatus: 'ready',
    disabledReason: 'Ready',
    lineColor,
    lineInputMode: 'fold-line',
  };
}

function commandAction(
  command: OristudioCpCommandDefinition,
  overrides: Partial<OristudioCpCommandActionDefinition> = {}
): OristudioCpCommandActionDefinition {
  return {
    id: actionIdForCommand(command),
    kind: 'command',
    label: command.label,
    group: command.group,
    placement: command.placement,
    icon: command.icon,
    upstreamAction: command.upstream,
    tooltip: command.tooltip,
    uiStatus: command.uiStatus,
    disabledReason: command.disabledReason,
    shortcut: command.shortcut,
    command,
    operationId: command.operationId,
    toolSteps: command.toolSteps,
    inputMode: command.inputMode,
    lineInputMode: 'fold-line',
    repeatable:
      command.operationId === 'DrawCreaseFree' ||
      command.operationId === 'DrawCreaseRestricted' ||
      command.operationId === 'CreaseSelect' ||
      command.operationId === 'CreaseUnselect' ||
      command.operationId === 'SelectPolygon' ||
      command.operationId === 'UnselectPolygon' ||
      command.operationId === 'SelectLineIntersecting' ||
      command.operationId === 'UnselectLineIntersecting' ||
      command.operationId === 'SelectLasso' ||
      command.operationId === 'UnselectLasso',
    ...overrides,
  };
}

function actionIdForCommand(command: OristudioCpCommandDefinition): OristudioCpActionId {
  return `cp.action.${command.id.replace(/^cp\./u, '')}`;
}

export function cpActionsForGroup(group: OristudioCpActionGroupId): OristudioCpActionDefinition[] {
  return sortActionsForRail(ORISTUDIO_CP_ACTIONS.filter((action) => action.group === group));
}

export function cpActionById(
  actionId: OristudioCpActionId
): OristudioCpActionDefinition | undefined {
  return ORISTUDIO_CP_ACTIONS.find((action) => action.id === actionId);
}

export function cpActionByOperation(
  operationId: OristudioCpOperationId
): OristudioCpCommandActionDefinition | undefined {
  return ORISTUDIO_CP_ACTIONS.find(
    (action): action is OristudioCpCommandActionDefinition =>
      action.kind === 'command' && action.operationId === operationId
  );
}

export function cpRailActions(): OristudioCpActionDefinition[] {
  return sortActionsForRail(
    ORISTUDIO_CP_ACTIONS.filter(
      (action) => action.placement === 'left-rail' || action.placement === 'left-rail-overflow'
    )
  );
}

function sortActionsForRail(
  actions: OristudioCpActionDefinition[]
): OristudioCpActionDefinition[] {
  return [...actions].sort((a, b) => {
    const groupA = actionGroupOrder(a.group);
    const groupB = actionGroupOrder(b.group);
    if (groupA !== groupB) return groupA - groupB;
    return actionOrder(a) - actionOrder(b);
  });
}

function actionGroupOrder(group: OristudioCpActionGroupId): number {
  return ORISTUDIO_CP_ACTION_GROUPS.find((definition) => definition.id === group)?.order ?? 999;
}

function actionOrder(action: OristudioCpActionDefinition): number {
  return action.railOrder ?? 10_000;
}
