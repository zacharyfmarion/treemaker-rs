import type {
  OristudioCpDocumentSnapshot,
  OristudioCpLineSegment,
} from '../engine/oristudioCpTypes';
import type { Point } from './geometry';
import type { OristudioCpSelection } from './creasePatternViewport';

const DEFAULT_PASTE_OFFSET = 8;
const ROTATION_SNAP_DEGREES = 22.5;

export interface CpLineSelectionBounds {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
  width: number;
  height: number;
  center: Point;
}

export interface CpLineClipboardPayload {
  kind: 'cp-lines';
  lines: OristudioCpLineSegment[];
  bounds: CpLineSelectionBounds;
}

export type CpSelectionTransform =
  | { kind: 'rotate'; angleDegrees: number; center?: Point }
  | { kind: 'flip-horizontal'; center?: Point; swapMountainValley?: boolean }
  | { kind: 'flip-vertical'; center?: Point; swapMountainValley?: boolean };

export function selectedCpLineSegments(
  document: OristudioCpDocumentSnapshot | null | undefined,
  selection: OristudioCpSelection
): OristudioCpLineSegment[] {
  if (!document || selection.lines.length === 0) return [];
  return selection.lines
    .map((id) => document.crease_pattern.line_segments[id - 1])
    .filter((line): line is OristudioCpLineSegment => Boolean(line))
    .map(cloneCpLineSegment);
}

export function buildCpLineClipboardPayload(
  document: OristudioCpDocumentSnapshot | null | undefined,
  selection: OristudioCpSelection
): CpLineClipboardPayload | null {
  const lines = selectedCpLineSegments(document, selection);
  const bounds = cpLineSelectionBounds(lines);
  return bounds ? { kind: 'cp-lines', lines, bounds } : null;
}

export function cpLineSelectionBounds(
  lines: readonly OristudioCpLineSegment[]
): CpLineSelectionBounds | null {
  if (lines.length === 0) return null;
  const points = lines.flatMap((line) => [line.a, line.b]);
  const minX = Math.min(...points.map((point) => point.x));
  const maxX = Math.max(...points.map((point) => point.x));
  const minY = Math.min(...points.map((point) => point.y));
  const maxY = Math.max(...points.map((point) => point.y));
  return {
    minX,
    minY,
    maxX,
    maxY,
    width: maxX - minX,
    height: maxY - minY,
    center: {
      x: (minX + maxX) / 2,
      y: (minY + maxY) / 2,
    },
  };
}

export function offsetCpLineSegmentsForPaste(
  lines: readonly OristudioCpLineSegment[],
  pasteCount: number
): OristudioCpLineSegment[] {
  const offset = DEFAULT_PASTE_OFFSET * ((pasteCount % 6) + 1);
  return lines.map((line) =>
    transformLinePoints(line, (point) => ({ x: point.x + offset, y: point.y - offset }))
  );
}

export function transformCpLineSegments(
  lines: readonly OristudioCpLineSegment[],
  transform: CpSelectionTransform
): OristudioCpLineSegment[] {
  const bounds = cpLineSelectionBounds(lines);
  if (!bounds) return [];
  const center = transform.center ?? bounds.center;

  if (transform.kind === 'rotate') {
    const radians = (transform.angleDegrees * Math.PI) / 180;
    const cos = Math.cos(radians);
    const sin = Math.sin(radians);
    return lines.map((line) =>
      transformLinePoints(line, (point) => {
        const x = point.x - center.x;
        const y = point.y - center.y;
        return {
          x: center.x + x * cos - y * sin,
          y: center.y + x * sin + y * cos,
        };
      })
    );
  }

  const swap = transform.swapMountainValley ?? false;
  return lines.map((line) => {
    const transformed = transformLinePoints(line, (point) =>
      transform.kind === 'flip-horizontal'
        ? { x: center.x * 2 - point.x, y: point.y }
        : { x: point.x, y: center.y * 2 - point.y }
    );
    return swap ? swapMountainValleyLine(transformed) : transformed;
  });
}

export function snapRotationDegrees(angleDegrees: number): number {
  return Math.round(angleDegrees / ROTATION_SNAP_DEGREES) * ROTATION_SNAP_DEGREES;
}

export function rotationAngleFromCenter(center: Point, point: Point): number {
  return (Math.atan2(point.y - center.y, point.x - center.x) * 180) / Math.PI;
}

export function cpSelectionTransformLabel(transform: CpSelectionTransform): string {
  switch (transform.kind) {
    case 'flip-horizontal':
      return 'Flip CP selection horizontal';
    case 'flip-vertical':
      return 'Flip CP selection vertical';
    case 'rotate':
      return `Rotate CP selection ${formatAngle(transform.angleDegrees)}`;
  }
}

function formatAngle(angleDegrees: number): string {
  const rounded = Math.round(angleDegrees * 100) / 100;
  return `${rounded} deg`;
}

function transformLinePoints(
  line: OristudioCpLineSegment,
  transform: (point: Point) => Point
): OristudioCpLineSegment {
  return {
    ...cloneCpLineSegment(line),
    a: transform(line.a),
    b: transform(line.b),
  };
}

function cloneCpLineSegment(line: OristudioCpLineSegment): OristudioCpLineSegment {
  return {
    ...line,
    a: { ...line.a },
    b: { ...line.b },
    customized_color: { ...line.customized_color },
  };
}

function swapMountainValleyLine(line: OristudioCpLineSegment): OristudioCpLineSegment {
  if (line.color === 'Red1') return { ...line, color: 'Blue2' };
  if (line.color === 'Blue2') return { ...line, color: 'Red1' };
  return line;
}
