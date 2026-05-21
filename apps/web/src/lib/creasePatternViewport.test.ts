import { describe, expect, it } from 'vitest';
import type { OristudioCpDocumentSnapshot } from '../engine/oristudioCpTypes';
import {
  cpLineAssignmentLabel,
  cpLineColorClass,
  cpSelectionSize,
  cpSvgPointToModel,
  getCpGridLines,
  getEditableCpModelBounds,
  modelPointToCpSvg,
  nearestCpSnapTarget,
  toggleCpSelectionList,
} from './creasePatternViewport';

const document: OristudioCpDocumentSnapshot = {
  title: 'fixture',
  metadata: {},
  crease_pattern: {
    line_segments: [
      {
        a: { x: 0, y: 0 },
        b: { x: 10, y: 0 },
        active: 'Inactive0',
        color: 'Red1',
        selected: 0,
        customized: 0,
        customized_color: { red: 100, green: 200, blue: 200 },
      },
      {
        a: { x: 0, y: 0 },
        b: { x: 0, y: 10 },
        active: 'Inactive0',
        color: 'Blue2',
        selected: 0,
        customized: 0,
        customized_color: { red: 100, green: 200, blue: 200 },
      },
    ],
    circles: [],
    points: [{ x: 5, y: 5 }],
    aux_line_segments: [],
    texts: [],
    grid: {
      interval_grid_size: 2,
      grid_size: 10,
      grid_xa: 1,
      grid_xb: 0,
      grid_xc: 1,
      grid_ya: 1,
      grid_yb: 0,
      grid_yc: 1,
      grid_angle: 90,
      base_state: 'WithinPaper',
      vertical_scale_position: 0,
      horizontal_scale_position: 0,
      draw_diagonal_gridlines: false,
    },
  },
};

describe('crease pattern viewport helpers', () => {
  it('computes padded model bounds and reversible SVG mapping', () => {
    const bounds = getEditableCpModelBounds(document);
    expect(bounds.minX).toBeLessThan(0);
    expect(bounds.maxY).toBeGreaterThan(10);

    const svg = modelPointToCpSvg({ x: 5, y: 5 }, bounds);
    const model = cpSvgPointToModel(svg, bounds);
    expect(model.x).toBeCloseTo(5);
    expect(model.y).toBeCloseTo(5);
  });

  it('generates bounded grid lines with major intervals', () => {
    const bounds = getEditableCpModelBounds(document);
    const lines = getCpGridLines(bounds, 10, 2);

    expect(lines).toHaveLength(22);
    expect(lines.filter((line) => line.major)).toHaveLength(12);
  });

  it('finds nearest snap candidates without mutating selection state', () => {
    const bounds = getEditableCpModelBounds(document);

    expect(
      nearestCpSnapTarget(document, { x: 0.03, y: 0.02 }, bounds, {
        gridVisible: true,
        snapToGrid: true,
        snapToVertices: true,
        snapToLines: true,
      })
    ).toMatchObject({ kind: 'line', label: 'line 1' });

    expect(toggleCpSelectionList([2], 1)).toEqual([1, 2]);
    expect(toggleCpSelectionList([1, 2], 1)).toEqual([2]);
    expect(
      cpSelectionSize({ lines: [1, 2], points: [1], circles: [], texts: [], faces: [] })
    ).toBe(3);
  });

  it('maps Oriedita line colors to existing CP render classes', () => {
    expect(cpLineColorClass('Red1', 'mvf')).toBe('crease crease--fold-mountain');
    expect(cpLineColorClass('Blue2', 'mvf')).toBe('crease crease--fold-valley');
    expect(cpLineColorClass('Cyan3', 'mvf')).toBe('crease crease--fold-flat');
    expect(cpLineColorClass('Red1', 'agrh')).toBe('crease crease--kind-axial');
    expect(cpLineAssignmentLabel('Black0')).toBe('edge');
  });
});
