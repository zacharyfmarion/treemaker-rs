import type {
  OristudioCpDocumentSnapshot,
  OristudioCpLineSegment,
} from '../engine/oristudioCpTypes';
import type { Point, PlotRect } from './geometry';

export const CP_VIEWBOX_SIZE = 720;
export const CP_WORLD_RECT: PlotRect = { x: 0, y: 0, width: CP_VIEWBOX_SIZE, height: CP_VIEWBOX_SIZE };
export const CP_PAPER_RECT: PlotRect = { x: 66, y: 54, width: 588, height: 588 };
export const CP_PAPER_SHADOW_RECT: PlotRect = { x: 56, y: 44, width: 608, height: 608 };

const MODEL_PADDING_RATIO = 0.04;
const DEFAULT_SPAN = 1;
const MAX_GRID_LINES = 80;

export interface OristudioCpSelection {
  lines: number[];
  points: number[];
  circles: number[];
  texts: number[];
  faces: number[];
}

export interface OristudioCpViewportOptions {
  gridVisible: boolean;
  snapToGrid: boolean;
  snapToVertices: boolean;
  snapToLines: boolean;
}

export type OristudioCpViewportOptionKey = keyof OristudioCpViewportOptions;

export const EMPTY_ORISTUDIO_CP_SELECTION: OristudioCpSelection = {
  lines: [],
  points: [],
  circles: [],
  texts: [],
  faces: [],
};

export const DEFAULT_ORISTUDIO_CP_VIEWPORT_OPTIONS: OristudioCpViewportOptions = {
  gridVisible: true,
  snapToGrid: true,
  snapToVertices: true,
  snapToLines: true,
};

export interface CpModelBounds {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
  spanX: number;
  spanY: number;
}

export interface CpGridLine {
  id: string;
  a: Point;
  b: Point;
  major: boolean;
}

export interface CpSnapTarget {
  point: Point;
  kind: 'grid' | 'vertex' | 'point' | 'line';
  label: string;
  distance: number;
}

export function emptyOristudioCpSelection(): OristudioCpSelection {
  return { ...EMPTY_ORISTUDIO_CP_SELECTION };
}

export function cpSelectionSize(selection: OristudioCpSelection): number {
  return (
    selection.lines.length +
    selection.points.length +
    selection.circles.length +
    selection.texts.length +
    selection.faces.length
  );
}

export function toggleCpSelectionList(ids: number[], id: number): number[] {
  return ids.includes(id)
    ? ids.filter((selectedId) => selectedId !== id)
    : Array.from(new Set([...ids, id])).sort((a, b) => a - b);
}

export function getEditableCpModelBounds(
  document: OristudioCpDocumentSnapshot | null | undefined
): CpModelBounds {
  const points: Point[] = [];
  if (document) {
    for (const segment of document.crease_pattern.line_segments) {
      points.push(segment.a, segment.b);
    }
    for (const segment of document.crease_pattern.aux_line_segments) {
      points.push(segment.a, segment.b);
    }
    points.push(...document.crease_pattern.points);
    for (const circle of document.crease_pattern.circles) {
      points.push(
        { x: circle.x - circle.r, y: circle.y - circle.r },
        { x: circle.x + circle.r, y: circle.y + circle.r }
      );
    }
    for (const text of document.crease_pattern.texts) {
      points.push({ x: textCoordinate(text.x), y: textCoordinate(text.y) });
    }
  }

  if (points.length === 0) {
    return boundsFromMinMax(0, 0, DEFAULT_SPAN, DEFAULT_SPAN);
  }

  let minX = Number.POSITIVE_INFINITY;
  let minY = Number.POSITIVE_INFINITY;
  let maxX = Number.NEGATIVE_INFINITY;
  let maxY = Number.NEGATIVE_INFINITY;
  for (const point of points) {
    minX = Math.min(minX, point.x);
    minY = Math.min(minY, point.y);
    maxX = Math.max(maxX, point.x);
    maxY = Math.max(maxY, point.y);
  }

  const span = Math.max(maxX - minX, maxY - minY, DEFAULT_SPAN);
  const padding = span * MODEL_PADDING_RATIO;
  return boundsFromMinMax(minX - padding, minY - padding, maxX + padding, maxY + padding);
}

export function modelPointToCpSvg(
  point: Point,
  bounds: CpModelBounds,
  rect: PlotRect = CP_PAPER_RECT
): Point {
  return {
    x: rect.x + ((point.x - bounds.minX) / bounds.spanX) * rect.width,
    y: rect.y + ((bounds.maxY - point.y) / bounds.spanY) * rect.height,
  };
}

export function cpSvgPointToModel(
  point: Point,
  bounds: CpModelBounds,
  rect: PlotRect = CP_PAPER_RECT
): Point {
  return {
    x: bounds.minX + ((point.x - rect.x) / rect.width) * bounds.spanX,
    y: bounds.maxY - ((point.y - rect.y) / rect.height) * bounds.spanY,
  };
}

export function cpLineColorClass(color: string, mode: 'mvf' | 'agrh'): string {
  if (mode === 'agrh') return 'crease crease--kind-axial';
  switch (color) {
    case 'Red1':
      return 'crease crease--fold-mountain';
    case 'Blue2':
      return 'crease crease--fold-valley';
    case 'Black0':
      return 'crease crease--fold-border';
    default:
      return 'crease crease--fold-flat';
  }
}

export function cpLineAssignmentLabel(color: string): string {
  switch (color) {
    case 'Red1':
      return 'mountain';
    case 'Blue2':
      return 'valley';
    case 'Black0':
      return 'edge';
    default:
      return 'auxiliary';
  }
}

export function getCpGridLines(
  bounds: CpModelBounds,
  gridSize: number,
  intervalSize: number
): CpGridLine[] {
  const divisions = Math.min(MAX_GRID_LINES, Math.max(1, gridSize));
  const interval = Math.max(1, intervalSize);
  const lines: CpGridLine[] = [];

  for (let index = 0; index <= divisions; index += 1) {
    const x = bounds.minX + (bounds.spanX * index) / divisions;
    const y = bounds.minY + (bounds.spanY * index) / divisions;
    const major = index % interval === 0;
    lines.push({
      id: `x-${index}`,
      a: { x, y: bounds.minY },
      b: { x, y: bounds.maxY },
      major,
    });
    lines.push({
      id: `y-${index}`,
      a: { x: bounds.minX, y },
      b: { x: bounds.maxX, y },
      major,
    });
  }

  return lines;
}

export function nearestCpSnapTarget(
  document: OristudioCpDocumentSnapshot,
  point: Point,
  bounds: CpModelBounds,
  options: OristudioCpViewportOptions,
  maxDistance = Math.max(bounds.spanX, bounds.spanY) * 0.015
): CpSnapTarget | null {
  let best: CpSnapTarget | null = null;
  const consider = (target: CpSnapTarget) => {
    if (target.distance > maxDistance) return;
    if (!best || target.distance < best.distance) best = target;
  };

  if (options.snapToVertices) {
    document.crease_pattern.line_segments.forEach((segment, index) => {
      consider(pointTarget(segment.a, point, 'vertex', `line ${index + 1} start`));
      consider(pointTarget(segment.b, point, 'vertex', `line ${index + 1} end`));
    });
    document.crease_pattern.points.forEach((candidate, index) => {
      consider(pointTarget(candidate, point, 'point', `point ${index + 1}`));
    });
  }

  if (options.snapToLines) {
    document.crease_pattern.line_segments.forEach((segment, index) => {
      const projected = closestPointOnSegment(point, segment);
      consider(pointTarget(projected, point, 'line', `line ${index + 1}`));
    });
  }

  if (options.snapToGrid) {
    const gridSize = Math.max(1, document.crease_pattern.grid.grid_size);
    const gridPoint = {
      x: snapCoordinate(point.x, bounds.minX, bounds.spanX / gridSize),
      y: snapCoordinate(point.y, bounds.minY, bounds.spanY / gridSize),
    };
    consider(pointTarget(gridPoint, point, 'grid', 'grid'));
  }

  return best;
}

export function textCoordinate(value: number | { 0: number }): number {
  return typeof value === 'number' ? value : value[0];
}

function boundsFromMinMax(minX: number, minY: number, maxX: number, maxY: number): CpModelBounds {
  return {
    minX,
    minY,
    maxX,
    maxY,
    spanX: Math.max(DEFAULT_SPAN * 1e-6, maxX - minX),
    spanY: Math.max(DEFAULT_SPAN * 1e-6, maxY - minY),
  };
}

function pointTarget(
  candidate: Point,
  point: Point,
  kind: CpSnapTarget['kind'],
  label: string
): CpSnapTarget {
  return {
    point: candidate,
    kind,
    label,
    distance: distance(candidate, point),
  };
}

function closestPointOnSegment(point: Point, segment: OristudioCpLineSegment): Point {
  const dx = segment.b.x - segment.a.x;
  const dy = segment.b.y - segment.a.y;
  const lengthSquared = dx * dx + dy * dy;
  if (lengthSquared === 0) return segment.a;
  const t = Math.max(
    0,
    Math.min(1, ((point.x - segment.a.x) * dx + (point.y - segment.a.y) * dy) / lengthSquared)
  );
  return {
    x: segment.a.x + dx * t,
    y: segment.a.y + dy * t,
  };
}

function snapCoordinate(value: number, origin: number, step: number): number {
  if (step <= 0) return value;
  return origin + Math.round((value - origin) / step) * step;
}

function distance(a: Point, b: Point): number {
  return Math.hypot(a.x - b.x, a.y - b.y);
}
