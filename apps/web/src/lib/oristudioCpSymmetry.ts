import type {
  OristudioCpCircle,
  OristudioCpCommandPayload,
  OristudioCpDocumentSnapshot,
  OristudioCpLineSegment,
} from '../engine/oristudioCpTypes';
import type { Point } from './geometry';
import type { CpModelBounds } from './creasePatternViewport';
import {
  reflectPointAcrossSymmetryAxis,
  type SymmetryAxis,
} from './symmetryAuthoring';

export type OristudioCpSymmetryPreset = 'none' | 'book' | 'diagonal' | 'custom';

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
  document: OristudioCpDocumentSnapshot,
  payload: OristudioCpCommandPayload,
  symmetry: OristudioCpSymmetryState
): OristudioCpCommandPayload[] {
  if (!symmetry.enabled) return [payload];

  const lineIds = payload.line_ids ?? [];
  const circleIds = payload.circle_ids ?? [];
  const mirroredLineIds = mirroredCpLineIds(document, lineIds, symmetry.axis);
  const mirroredCircleIds = mirroredCpCircleIds(document, circleIds, symmetry.axis);
  const primaryPayload = mirroredLineIds.length > 0 || mirroredCircleIds.length > 0
    ? {
        ...payload,
        line_ids:
          lineIds.length > 0
            ? sortedUniqueNumbers([...lineIds, ...mirroredLineIds])
            : payload.line_ids,
        circle_ids:
          circleIds.length > 0
            ? sortedUniqueNumbers([...circleIds, ...mirroredCircleIds])
            : payload.circle_ids,
      }
    : payload;

  if (!payload.points || payload.points.length === 0) {
    return [primaryPayload];
  }

  const reflectedPoints = payload.points.map((point) =>
    reflectPointAcrossSymmetryAxis(point, symmetry.axis)
  );
  if (samePointList(payload.points, reflectedPoints)) {
    return [primaryPayload];
  }

  if (
    (lineIds.length > 0 && mirroredLineIds.length === 0) ||
    (circleIds.length > 0 && mirroredCircleIds.length === 0)
  ) {
    return [primaryPayload];
  }

  return [
    primaryPayload,
    {
      ...payload,
      points: reflectedPoints,
      line_ids: lineIds.length > 0 ? mirroredLineIds : payload.line_ids,
      circle_ids: circleIds.length > 0 ? mirroredCircleIds : payload.circle_ids,
      replace_selection: payload.replace_selection ? false : payload.replace_selection,
    },
  ];
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
  const lines = document.crease_pattern.line_segments;
  for (const id of lineIds) {
    const line = lines[id - 1];
    if (!line) continue;
    const reflected = reflectCpLineSegment(line, axis);
    lines.forEach((candidate, index) => {
      const candidateId = index + 1;
      if (candidateId === id || ids.has(candidateId)) return;
      if (sameSegment(candidate, reflected)) ids.add(candidateId);
    });
  }
  return Array.from(ids).sort((a, b) => a - b);
}

export function mirroredCpCircleIds(
  document: OristudioCpDocumentSnapshot,
  circleIds: readonly number[],
  axis: SymmetryAxis
): number[] {
  const ids = new Set<number>();
  const circles = document.crease_pattern.circles;
  for (const id of circleIds) {
    const circle = circles[id - 1];
    if (!circle) continue;
    const reflected = reflectCpCircle(circle, axis);
    circles.forEach((candidate, index) => {
      const candidateId = index + 1;
      if (candidateId === id || ids.has(candidateId)) return;
      if (sameCircle(candidate, reflected)) ids.add(candidateId);
    });
  }
  return Array.from(ids).sort((a, b) => a - b);
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

function normalizeAngle(angle: number): number {
  return ((angle % 180) + 180) % 180;
}

function numberOr(value: unknown, fallback: number): number {
  return typeof value === 'number' && Number.isFinite(value) ? value : fallback;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}
