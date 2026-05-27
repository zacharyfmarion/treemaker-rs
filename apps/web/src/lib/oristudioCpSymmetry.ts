import type {
  OristudioCpCircle,
  OristudioCpCommandPayload,
  OristudioCpDocumentSnapshot,
  OristudioCpLineSegment,
  OristudioCpTextElement,
} from '../engine/oristudioCpTypes';
import type { Point } from './geometry';
import type { CpModelBounds } from './creasePatternViewport';
import type { OristudioCpOperationId } from './oristudioCpCommands';
import {
  reflectPointAcrossSymmetryAxis,
  type SymmetryAxis,
} from './symmetryAuthoring';

export type OristudioCpSymmetryPreset = 'none' | 'book' | 'diagonal' | 'custom';
export type OristudioCpSymmetryPolicy =
  | 'point-payloads'
  | 'selection-payloads'
  | 'unordered-entity-selection'
  | 'ordered-entity-payloads'
  | 'text-payloads'
  | 'run-once';

export interface OristudioCpSymmetryState {
  enabled: boolean;
  showAxis: boolean;
  preset: OristudioCpSymmetryPreset;
  axis: SymmetryAxis;
}

const DEFAULT_CP_AXIS: SymmetryAxis = {
  loc: { x: 0, y: 0 },
  angle: 90,
};

const POINT_TOLERANCE = 1e-7;

export const DEFAULT_ORISTUDIO_CP_SYMMETRY: OristudioCpSymmetryState = {
  enabled: false,
  showAxis: true,
  preset: 'none',
  axis: DEFAULT_CP_AXIS,
};

export const ORISTUDIO_CP_SYMMETRY_POLICIES: Partial<
  Record<OristudioCpOperationId, OristudioCpSymmetryPolicy>
> = {
  DrawCreaseFree: 'point-payloads',
  LineSegmentDelete: 'unordered-entity-selection',
  ChangeCreaseType: 'unordered-entity-selection',
  LengthenCrease: 'ordered-entity-payloads',
  SquareBisector: 'ordered-entity-payloads',
  Inward: 'point-payloads',
  PerpendicularDraw: 'point-payloads',
  SymmetricDraw: 'point-payloads',
  DrawCreaseRestricted: 'point-payloads',
  DrawCreaseSymmetric: 'point-payloads',
  DrawCreaseAngleRestricted: 'point-payloads',
  DrawPoint: 'point-payloads',
  DeletePoint: 'point-payloads',
  AngleSystem: 'point-payloads',
  DrawCreaseAngleRestricted3: 'point-payloads',
  CreaseSelect: 'selection-payloads',
  CreaseUnselect: 'selection-payloads',
  CreaseMove: 'point-payloads',
  CreaseCopy: 'point-payloads',
  CreaseMakeMountain: 'unordered-entity-selection',
  CreaseMakeValley: 'unordered-entity-selection',
  CreaseMakeEdge: 'unordered-entity-selection',
  LineSegmentDivision: 'ordered-entity-payloads',
  LineSegmentRatioSet: 'ordered-entity-payloads',
  PolygonSetNoCorners: 'point-payloads',
  CreaseAdvanceType: 'unordered-entity-selection',
  CreaseMove4p: 'point-payloads',
  CreaseCopy4p: 'point-payloads',
  FishBoneDraw: 'point-payloads',
  CreaseMakeMv: 'point-payloads',
  DoubleSymmetricDraw: 'point-payloads',
  CreasesAlternateMv: 'point-payloads',
  DrawCreaseAngleRestricted5: 'point-payloads',
  VertexMakeAngularlyFlatFoldable: 'point-payloads',
  FoldableLineInput: 'point-payloads',
  ParallelDraw: 'point-payloads',
  VertexDeleteOnCrease: 'point-payloads',
  CircleDraw: 'point-payloads',
  CircleDrawThreePoint: 'point-payloads',
  CircleDrawSeparate: 'point-payloads',
  CircleDrawTangentLine: 'ordered-entity-payloads',
  CircleDrawInverted: 'ordered-entity-payloads',
  CircleDrawFree: 'point-payloads',
  CircleDrawConcentric: 'ordered-entity-payloads',
  CircleDrawConcentricSelect: 'ordered-entity-payloads',
  CircleDrawConcentricTwoCircleSelect: 'ordered-entity-payloads',
  ParallelDrawWidth: 'ordered-entity-payloads',
  ContinuousSymmetricDraw: 'point-payloads',
  DisplayLengthBetweenPoints1: 'run-once',
  DisplayLengthBetweenPoints2: 'run-once',
  DisplayAngleBetweenThreePoints1: 'run-once',
  DisplayAngleBetweenThreePoints2: 'run-once',
  DisplayAngleBetweenThreePoints3: 'run-once',
  CreaseToggleMv: 'unordered-entity-selection',
  CircleChangeColor: 'unordered-entity-selection',
  CreaseMakeAux: 'unordered-entity-selection',
  OperationFrameCreate: 'run-once',
  VoronoiCreate: 'point-payloads',
  FlatFoldableCheck: 'run-once',
  CreaseDeleteOverlapping: 'point-payloads',
  CreaseDeleteIntersecting: 'point-payloads',
  SelectPolygon: 'selection-payloads',
  UnselectPolygon: 'selection-payloads',
  SelectLineIntersecting: 'selection-payloads',
  UnselectLineIntersecting: 'selection-payloads',
  LengthenCreaseSameColor: 'ordered-entity-payloads',
  FoldableLineDraw: 'point-payloads',
  ReplaceLineTypeSelect: 'unordered-entity-selection',
  DeleteLineTypeSelect: 'unordered-entity-selection',
  SelectLasso: 'selection-payloads',
  UnselectLasso: 'selection-payloads',
  Text: 'text-payloads',
  DrawBlintz: 'point-payloads',
  DrawFishBase: 'point-payloads',
  DrawDoveBase: 'point-payloads',
  DrawBirdBase: 'point-payloads',
  DrawFrogBase: 'point-payloads',
  Axiom5: 'point-payloads',
  Axiom7: 'point-payloads',
  FixInaccurate: 'unordered-entity-selection',
  CheckCamv: 'run-once',
  Check1: 'run-once',
  Check2: 'run-once',
  Check3: 'run-once',
  Check4: 'run-once',
  Fix1: 'run-once',
  Fix2: 'run-once',
  OrganizeCircles: 'run-once',
};

export function defaultOristudioCpSymmetry(): OristudioCpSymmetryState {
  return {
    ...DEFAULT_ORISTUDIO_CP_SYMMETRY,
    axis: { ...DEFAULT_ORISTUDIO_CP_SYMMETRY.axis, loc: { ...DEFAULT_ORISTUDIO_CP_SYMMETRY.axis.loc } },
  };
}

export function normalizeOristudioCpSymmetry(value: unknown): OristudioCpSymmetryState {
  if (!isRecord(value)) return defaultOristudioCpSymmetry();
  const axis = isRecord(value.axis) ? value.axis : {};
  const loc = isRecord(axis.loc) ? axis.loc : {};
  const preset = value.preset;
  return {
    enabled: value.enabled === true,
    showAxis: value.showAxis !== false,
    preset:
      preset === 'none' || preset === 'book' || preset === 'diagonal' || preset === 'custom'
        ? preset
        : 'none',
    axis: {
      loc: {
        x: numberOr(loc.x, DEFAULT_CP_AXIS.loc.x),
        y: numberOr(loc.y, DEFAULT_CP_AXIS.loc.y),
      },
      angle: numberOr(axis.angle, DEFAULT_CP_AXIS.angle),
    },
  };
}

export function cpSymmetryPresetAxis(
  preset: Exclude<OristudioCpSymmetryPreset, 'none' | 'custom'>,
  currentAngle = 90
): SymmetryAxis {
  if (preset === 'diagonal') {
    const normalized = normalizeAngle(currentAngle);
    return {
      loc: { x: 0, y: 0 },
      angle: Math.abs(normalized - 45) < Math.abs(normalized - 135) ? 45 : 135,
    };
  }
  const normalized = normalizeAngle(currentAngle);
  return {
    loc: { x: 0, y: 0 },
    angle: Math.abs(normalized - 0) < Math.abs(normalized - 90) ? 0 : 90,
  };
}

export function nextCpSymmetryPresetAxis(
  symmetry: OristudioCpSymmetryState
): SymmetryAxis | null {
  if (symmetry.preset === 'book') {
    return {
      loc: { x: 0, y: 0 },
      angle: Math.abs(normalizeAngle(symmetry.axis.angle)) < 45 ? 90 : 0,
    };
  }
  if (symmetry.preset === 'diagonal') {
    return {
      loc: { x: 0, y: 0 },
      angle: Math.abs(normalizeAngle(symmetry.axis.angle) - 45) < 45 ? 135 : 45,
    };
  }
  return null;
}

export function axisFromTwoPoints(a: Point, b: Point): SymmetryAxis | null {
  if (samePoint(a, b)) return null;
  return {
    loc: a,
    angle: (Math.atan2(b.y - a.y, b.x - a.x) * 180) / Math.PI,
  };
}

export function reflectCpLineSegment(
  segment: OristudioCpLineSegment,
  axis: SymmetryAxis
): OristudioCpLineSegment {
  return {
    ...segment,
    a: reflectPointAcrossSymmetryAxis(segment.a, axis),
    b: reflectPointAcrossSymmetryAxis(segment.b, axis),
  };
}

export function reflectCpCircle(circle: OristudioCpCircle, axis: SymmetryAxis): OristudioCpCircle {
  const center = reflectPointAcrossSymmetryAxis({ x: circle.x, y: circle.y }, axis);
  return {
    ...circle,
    x: center.x,
    y: center.y,
  };
}

export function reflectedCpCommandPayloads(
  operationId: OristudioCpOperationId,
  document: OristudioCpDocumentSnapshot,
  payload: OristudioCpCommandPayload,
  symmetry: OristudioCpSymmetryState
): OristudioCpCommandPayload[] {
  if (!symmetry.enabled) return [payload];

  switch (ORISTUDIO_CP_SYMMETRY_POLICIES[operationId] ?? 'run-once') {
    case 'selection-payloads':
      return reflectedSelectionPayloads(document, payload, symmetry);
    case 'unordered-entity-selection':
      return [expandedEntitySelectionPayload(document, payload, symmetry.axis)];
    case 'ordered-entity-payloads':
      return reflectedOrderedEntityPayloads(document, payload, symmetry);
    case 'point-payloads':
      return reflectedPointPayloads(document, payload, symmetry);
    case 'text-payloads':
      return reflectedTextPayloads(document, payload, symmetry);
    case 'run-once':
      return [payload];
  }
  return [payload];
}

export function reflectedPreviewSegments(
  segments: OristudioCpLineSegment[],
  symmetry: OristudioCpSymmetryState
): OristudioCpLineSegment[] {
  if (!symmetry.enabled) return segments;
  return [...segments, ...segments.map((segment) => reflectCpLineSegment(segment, symmetry.axis))];
}

export function reflectedPreviewPoints(
  points: Point[],
  symmetry: OristudioCpSymmetryState
): Point[] {
  if (!symmetry.enabled) return points;
  return [...points, ...points.map((point) => reflectPointAcrossSymmetryAxis(point, symmetry.axis))];
}

export function reflectedPreviewCircles(
  circles: OristudioCpCircle[],
  symmetry: OristudioCpSymmetryState
): OristudioCpCircle[] {
  if (!symmetry.enabled) return circles;
  return [...circles, ...circles.map((circle) => reflectCpCircle(circle, symmetry.axis))];
}

export function cpSymmetryAxisLine(axis: SymmetryAxis, bounds: CpModelBounds): [Point, Point] {
  const radians = (axis.angle * Math.PI) / 180;
  const span = Math.hypot(bounds.spanX, bounds.spanY) * 2;
  const dx = Math.cos(radians) * span;
  const dy = Math.sin(radians) * span;
  return [
    { x: axis.loc.x - dx, y: axis.loc.y - dy },
    { x: axis.loc.x + dx, y: axis.loc.y + dy },
  ];
}

export function mirroredCpLineIds(
  document: OristudioCpDocumentSnapshot,
  lineIds: readonly number[],
  axis: SymmetryAxis
): number[] {
  const ids = new Set<number>();
  for (const id of lineIds) {
    const mirrored = mirroredCpLineId(document, id, axis);
    if (mirrored !== null && mirrored !== id) ids.add(mirrored);
  }
  return Array.from(ids).sort((a, b) => a - b);
}

export function mirroredCpLineIdsInOrder(
  document: OristudioCpDocumentSnapshot,
  lineIds: readonly number[],
  axis: SymmetryAxis
): number[] | null {
  const mirrored = lineIds.map((id) => mirroredCpLineId(document, id, axis));
  if (mirrored.some((id) => id === null)) return null;
  return mirrored as number[];
}

export function mirroredCpCircleIds(
  document: OristudioCpDocumentSnapshot,
  circleIds: readonly number[],
  axis: SymmetryAxis
): number[] {
  const ids = new Set<number>();
  for (const id of circleIds) {
    const mirrored = mirroredCpCircleId(document, id, axis);
    if (mirrored !== null && mirrored !== id) ids.add(mirrored);
  }
  return Array.from(ids).sort((a, b) => a - b);
}

export function mirroredCpCircleIdsInOrder(
  document: OristudioCpDocumentSnapshot,
  circleIds: readonly number[],
  axis: SymmetryAxis
): number[] | null {
  const mirrored = circleIds.map((id) => mirroredCpCircleId(document, id, axis));
  if (mirrored.some((id) => id === null)) return null;
  return mirrored as number[];
}

export function mirroredCpTextIds(
  document: OristudioCpDocumentSnapshot,
  textIds: readonly number[],
  axis: SymmetryAxis
): number[] {
  const ids = new Set<number>();
  for (const id of textIds) {
    const mirrored = mirroredCpTextId(document, id, axis);
    if (mirrored !== null && mirrored !== id) ids.add(mirrored);
  }
  return Array.from(ids).sort((a, b) => a - b);
}

export function mirroredCpTextIdsInOrder(
  document: OristudioCpDocumentSnapshot,
  textIds: readonly number[],
  axis: SymmetryAxis
): number[] | null {
  const mirrored = textIds.map((id) => mirroredCpTextId(document, id, axis));
  if (mirrored.some((id) => id === null)) return null;
  return mirrored as number[];
}

function reflectedSelectionPayloads(
  document: OristudioCpDocumentSnapshot,
  payload: OristudioCpCommandPayload,
  symmetry: OristudioCpSymmetryState
): OristudioCpCommandPayload[] {
  if (!payload.points || payload.points.length === 0) {
    return [expandedEntitySelectionPayload(document, payload, symmetry.axis)];
  }

  return reflectedPointPayloads(document, payload, symmetry);
}

function reflectedOrderedEntityPayloads(
  document: OristudioCpDocumentSnapshot,
  payload: OristudioCpCommandPayload,
  symmetry: OristudioCpSymmetryState
): OristudioCpCommandPayload[] {
  const mirroredPayload = mirroredEntityPayloadInOrder(document, payload, symmetry.axis);
  if (!mirroredPayload && hasEntityIds(payload)) return [payload];
  return reflectedPointPayloads(document, payload, symmetry, mirroredPayload ?? payload);
}

function reflectedPointPayloads(
  document: OristudioCpDocumentSnapshot,
  payload: OristudioCpCommandPayload,
  symmetry: OristudioCpSymmetryState,
  mirroredEntityPayload = hasEntityIds(payload)
    ? mirroredEntityPayloadInOrder(document, payload, symmetry.axis)
    : payload
): OristudioCpCommandPayload[] {
  if (!payload.points || payload.points.length === 0) {
    if (!hasEntityIds(payload) || !mirroredEntityPayload) return [payload];
    return uniquePayloads([payload, mirroredEntityPayload]);
  }

  if (!mirroredEntityPayload) return [payload];

  const reflectedPoints = payload.points.map((point) =>
    reflectPointAcrossSymmetryAxis(point, symmetry.axis)
  );
  const reflectedPayload = {
    ...mirroredEntityPayload,
    points: reflectedPoints,
    replace_selection: payload.replace_selection ? false : payload.replace_selection,
  };

  return uniquePayloads([payload, reflectedPayload]);
}

function reflectedTextPayloads(
  document: OristudioCpDocumentSnapshot,
  payload: OristudioCpCommandPayload,
  symmetry: OristudioCpSymmetryState
): OristudioCpCommandPayload[] {
  switch (payload.text_action) {
    case 'SetContent':
    case 'DeleteSelected':
      return [expandedEntitySelectionPayload(document, payload, symmetry.axis)];
    case 'Move':
      return reflectedOrderedEntityPayloads(document, payload, symmetry);
    case 'DeleteAt':
    case 'DeleteBox':
    case 'Create':
    case undefined:
      return reflectedPointPayloads(document, payload, symmetry);
  }
}

function expandedEntitySelectionPayload(
  document: OristudioCpDocumentSnapshot,
  payload: OristudioCpCommandPayload,
  axis: SymmetryAxis
): OristudioCpCommandPayload {
  const lineIds = payload.line_ids ?? [];
  const circleIds = payload.circle_ids ?? [];
  const textIds = payload.text_ids ?? [];
  const mirroredLineIds = mirroredCpLineIds(document, lineIds, axis);
  const mirroredCircleIds = mirroredCpCircleIds(document, circleIds, axis);
  const mirroredTextIds = mirroredCpTextIds(document, textIds, axis);

  if (
    mirroredLineIds.length === 0 &&
    mirroredCircleIds.length === 0 &&
    mirroredTextIds.length === 0
  ) {
    return payload;
  }

  return {
    ...payload,
    line_ids:
      lineIds.length > 0
        ? sortedUniqueNumbers([...lineIds, ...mirroredLineIds])
        : payload.line_ids,
    circle_ids:
      circleIds.length > 0
        ? sortedUniqueNumbers([...circleIds, ...mirroredCircleIds])
        : payload.circle_ids,
    text_ids:
      textIds.length > 0
        ? sortedUniqueNumbers([...textIds, ...mirroredTextIds])
        : payload.text_ids,
  };
}

function mirroredEntityPayloadInOrder(
  document: OristudioCpDocumentSnapshot,
  payload: OristudioCpCommandPayload,
  axis: SymmetryAxis
): OristudioCpCommandPayload | null {
  const lineIds = payload.line_ids ?? [];
  const circleIds = payload.circle_ids ?? [];
  const textIds = payload.text_ids ?? [];
  let mirroredPayload = payload;

  if (lineIds.length > 0) {
    const mirroredLineIds = mirroredCpLineIdsInOrder(document, lineIds, axis);
    if (!mirroredLineIds) return null;
    mirroredPayload = { ...mirroredPayload, line_ids: mirroredLineIds };
  }

  if (circleIds.length > 0) {
    const mirroredCircleIds = mirroredCpCircleIdsInOrder(document, circleIds, axis);
    if (!mirroredCircleIds) return null;
    mirroredPayload = { ...mirroredPayload, circle_ids: mirroredCircleIds };
  }

  if (textIds.length > 0) {
    const mirroredTextIds = mirroredCpTextIdsInOrder(document, textIds, axis);
    if (!mirroredTextIds) return null;
    mirroredPayload = { ...mirroredPayload, text_ids: mirroredTextIds };
  }

  return mirroredPayload;
}

function mirroredCpLineId(
  document: OristudioCpDocumentSnapshot,
  lineId: number,
  axis: SymmetryAxis
): number | null {
  const lines = document.crease_pattern.line_segments;
  const line = lines[lineId - 1];
  if (!line) return null;
  const reflected = reflectCpLineSegment(line, axis);
  if (sameSegment(line, reflected)) return lineId;
  return findMatchingId(lines, reflected, sameSegment);
}

function mirroredCpCircleId(
  document: OristudioCpDocumentSnapshot,
  circleId: number,
  axis: SymmetryAxis
): number | null {
  const circles = document.crease_pattern.circles;
  const circle = circles[circleId - 1];
  if (!circle) return null;
  const reflected = reflectCpCircle(circle, axis);
  if (sameCircle(circle, reflected)) return circleId;
  return findMatchingId(circles, reflected, sameCircle);
}

function mirroredCpTextId(
  document: OristudioCpDocumentSnapshot,
  textId: number,
  axis: SymmetryAxis
): number | null {
  const texts = document.crease_pattern.texts;
  const text = texts[textId - 1];
  if (!text) return null;
  const reflected = reflectCpText(text, axis);
  if (sameTextPosition(text, reflected)) return textId;
  return findMatchingId(
    texts,
    reflected,
    (candidate, target) => candidate.text === text.text && sameTextPosition(candidate, target)
  );
}

function reflectCpText(text: OristudioCpTextElement, axis: SymmetryAxis): OristudioCpTextElement {
  const position = textPosition(text);
  if (!position) return text;
  const reflected = reflectPointAcrossSymmetryAxis(position, axis);
  return {
    ...text,
    x: reflected.x,
    y: reflected.y,
  };
}

function findMatchingId<T>(
  values: readonly T[],
  target: T,
  matches: (candidate: T, target: T) => boolean
): number | null {
  const index = values.findIndex((candidate) => matches(candidate, target));
  return index >= 0 ? index + 1 : null;
}

function sameSegment(a: OristudioCpLineSegment, b: OristudioCpLineSegment): boolean {
  return (
    (samePoint(a.a, b.a) && samePoint(a.b, b.b)) ||
    (samePoint(a.a, b.b) && samePoint(a.b, b.a))
  );
}

function sameCircle(a: OristudioCpCircle, b: OristudioCpCircle): boolean {
  return (
    samePoint({ x: a.x, y: a.y }, { x: b.x, y: b.y }) &&
    Math.abs(a.r - b.r) <= POINT_TOLERANCE
  );
}

function sameTextPosition(a: OristudioCpTextElement, b: OristudioCpTextElement): boolean {
  const aPosition = textPosition(a);
  const bPosition = textPosition(b);
  return aPosition !== null && bPosition !== null && samePoint(aPosition, bPosition);
}

function samePointList(a: readonly Point[], b: readonly Point[]): boolean {
  if (a.length !== b.length) return false;
  return a.every((point, index) => {
    const other = b[index];
    return other ? samePoint(point, other) : false;
  });
}

function samePoint(a: Point, b: Point): boolean {
  return Math.hypot(a.x - b.x, a.y - b.y) <= POINT_TOLERANCE;
}

function sortedUniqueNumbers(values: number[]): number[] {
  return Array.from(new Set(values)).sort((a, b) => a - b);
}

function uniquePayloads(payloads: OristudioCpCommandPayload[]): OristudioCpCommandPayload[] {
  return payloads.filter((payload, index) =>
    payloads.findIndex((candidate) => samePayload(candidate, payload)) === index
  );
}

function samePayload(
  a: OristudioCpCommandPayload,
  b: OristudioCpCommandPayload
): boolean {
  return (
    sameNumberList(a.line_ids ?? [], b.line_ids ?? []) &&
    sameNumberList(a.circle_ids ?? [], b.circle_ids ?? []) &&
    sameNumberList(a.text_ids ?? [], b.text_ids ?? []) &&
    samePointList(a.points ?? [], b.points ?? []) &&
    a.replace_selection === b.replace_selection
  );
}

function sameNumberList(a: readonly number[], b: readonly number[]): boolean {
  if (a.length !== b.length) return false;
  return a.every((value, index) => value === b[index]);
}

function hasEntityIds(payload: OristudioCpCommandPayload): boolean {
  return (
    (payload.line_ids?.length ?? 0) > 0 ||
    (payload.circle_ids?.length ?? 0) > 0 ||
    (payload.text_ids?.length ?? 0) > 0
  );
}

function textPosition(text: OristudioCpTextElement): Point | null {
  const x = textCoordinate(text.x);
  const y = textCoordinate(text.y);
  if (x === null || y === null) return null;
  return { x, y };
}

function textCoordinate(value: number | { 0: number }): number | null {
  const coordinate = typeof value === 'number' ? value : value[0];
  return Number.isFinite(coordinate) ? coordinate : null;
}

function normalizeAngle(angle: number): number {
  return ((angle % 180) + 180) % 180;
}

function numberOr(value: unknown, fallback: number): number {
  return typeof value === 'number' && Number.isFinite(value) ? value : fallback;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}
