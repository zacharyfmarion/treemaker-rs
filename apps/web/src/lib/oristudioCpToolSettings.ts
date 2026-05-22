import type {
  OristudioCpCustomLineType,
  OristudioCpRgbColor,
} from '../engine/oristudioCpTypes';
import type {
  OristudioCpCommandDefinition,
  OristudioCpOperationId,
} from './oristudioCpCommands';

export type OristudioCpToolSettingGroup =
  | 'line-color'
  | 'angle-system'
  | 'division-count'
  | 'division-ratio'
  | 'replace-line-type'
  | 'delete-line-type'
  | 'fix-precision'
  | 'polygon-corners'
  | 'parallel-width'
  | 'candidate-choice'
  | 'line-select-help'
  | 'apply-lines'
  | 'measurement-readout'
  | 'custom-circle-color';

export interface OristudioCpRatioExpression {
  a: number;
  b: number;
  c: number;
  d: number;
  e: number;
  f: number;
}

export interface OristudioCpToolOptions {
  divisionCount: number;
  divisionRatio: OristudioCpRatioExpression;
  angleSystemDivider: number;
  angleSystemAngles: [number, number, number, number, number, number];
  customFromLineType: OristudioCpCustomLineType;
  customToLineType: OristudioCpCustomLineType;
  customLineType: OristudioCpCustomLineType;
  fixPrecision: number;
  fixPrecisionUseBp: boolean;
  fixPrecisionUse22_5: boolean;
  polygonCorners: number;
  parallelWidth: number;
  candidateIndex: number | null;
  customCircleColor: OristudioCpRgbColor;
}

export const DEFAULT_ORISTUDIO_CP_TOOL_OPTIONS: OristudioCpToolOptions = {
  divisionCount: 2,
  divisionRatio: {
    a: 1,
    b: 0,
    c: 0,
    d: 0,
    e: 1,
    f: 2,
  },
  angleSystemDivider: 8,
  angleSystemAngles: [40, 60, 80, 30, 50, 100],
  customFromLineType: 'Any',
  customToLineType: 'Edge',
  customLineType: 'Any',
  fixPrecision: 0.05,
  fixPrecisionUseBp: true,
  fixPrecisionUse22_5: true,
  polygonCorners: 5,
  parallelWidth: 1,
  candidateIndex: null,
  customCircleColor: { red: 100, green: 200, blue: 200 },
};

export const ORISTUDIO_CP_CUSTOM_LINE_TYPE_OPTIONS = [
  { value: 'Any', label: 'Any' },
  { value: 'Edge', label: 'Edge' },
  { value: 'MountainAndValley', label: 'M/V' },
  { value: 'Mountain', label: 'Mountain' },
  { value: 'Valley', label: 'Valley' },
  { value: 'Aux', label: 'Auxiliary' },
] as const satisfies readonly {
  value: OristudioCpCustomLineType;
  label: string;
}[];

export const ORISTUDIO_CP_REPLACE_TARGET_LINE_TYPE_OPTIONS =
  ORISTUDIO_CP_CUSTOM_LINE_TYPE_OPTIONS.filter(
    (option) => option.value !== 'Any' && option.value !== 'MountainAndValley'
  );

const LINE_COLOR_OPERATION_IDS = new Set<OristudioCpOperationId>([
  'CreaseMakeMv',
  'CreasesAlternateMv',
  'LengthenCrease',
  'DrawCreaseFree',
  'DrawCreaseRestricted',
  'DrawCreaseSymmetric',
  'DrawCreaseAngleRestricted',
  'DrawCreaseAngleRestricted3',
  'DrawCreaseAngleRestricted5',
  'SquareBisector',
  'Inward',
  'PerpendicularDraw',
  'SymmetricDraw',
  'FishBoneDraw',
  'DoubleSymmetricDraw',
  'VertexMakeAngularlyFlatFoldable',
  'FoldableLineInput',
  'ParallelDraw',
  'ParallelDrawWidth',
  'ContinuousSymmetricDraw',
  'FoldableLineDraw',
  'Axiom5',
  'Axiom7',
]);

const TOOL_SETTING_GROUPS_BY_OPERATION: Partial<
  Record<OristudioCpOperationId, readonly OristudioCpToolSettingGroup[]>
> = {
  AngleSystem: ['angle-system'],
  DrawCreaseAngleRestricted: ['angle-system', 'candidate-choice'],
  DrawCreaseAngleRestricted3: ['angle-system', 'candidate-choice'],
  DrawCreaseAngleRestricted5: ['angle-system', 'candidate-choice'],
  LineSegmentDivision: ['division-count'],
  LineSegmentRatioSet: ['division-ratio'],
  PolygonSetNoCorners: ['polygon-corners'],
  ParallelDrawWidth: ['parallel-width'],
  ReplaceLineTypeSelect: ['replace-line-type'],
  DeleteLineTypeSelect: ['delete-line-type'],
  FixInaccurate: ['fix-precision'],
  SelectLineIntersecting: ['line-select-help'],
  UnselectLineIntersecting: ['line-select-help'],
  CreaseDeleteIntersecting: ['line-select-help'],
  DisplayLengthBetweenPoints1: ['measurement-readout'],
  DisplayLengthBetweenPoints2: ['measurement-readout'],
  DisplayAngleBetweenThreePoints1: ['measurement-readout'],
  DisplayAngleBetweenThreePoints2: ['measurement-readout'],
  DisplayAngleBetweenThreePoints3: ['measurement-readout'],
  CircleChangeColor: ['custom-circle-color'],
  VoronoiCreate: ['apply-lines'],
  Axiom5: ['candidate-choice'],
  Axiom7: ['candidate-choice'],
};

export function cpToolSettingGroupsForOperation(
  operationId: OristudioCpOperationId
): readonly OristudioCpToolSettingGroup[] {
  const groups: OristudioCpToolSettingGroup[] = [];
  if (LINE_COLOR_OPERATION_IDS.has(operationId)) {
    groups.push('line-color');
  }
  groups.push(...(TOOL_SETTING_GROUPS_BY_OPERATION[operationId] ?? []));
  return groups;
}

export function cpToolSettingGroupsForCommand(
  command: OristudioCpCommandDefinition | null | undefined
): readonly OristudioCpToolSettingGroup[] {
  return command ? cpToolSettingGroupsForOperation(command.operationId) : [];
}

export function evaluateOrieditaRatioExpression(
  expression: OristudioCpRatioExpression
): { ratioS: number; ratioT: number } {
  return {
    ratioS: evaluateRatioPart(expression.a, expression.b, expression.c),
    ratioT: evaluateRatioPart(expression.d, expression.e, expression.f),
  };
}

function evaluateRatioPart(a: number, b: number, c: number): number {
  const linear = Number.isFinite(a) ? a : 0;
  const radical = Number.isFinite(b) ? b : 0;
  const radicand = Number.isFinite(c) ? Math.max(0, c) : 0;
  const value = linear + radical * Math.sqrt(radicand);
  return Number.isFinite(value) ? Math.max(0, value) : 0;
}
