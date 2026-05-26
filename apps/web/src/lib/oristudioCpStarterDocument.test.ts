import { describe, expect, it } from 'vitest';
import { ORIEDITA_PAPER_MAX, ORIEDITA_PAPER_MIN } from './creasePatternViewport';
import {
  STARTER_ORISTUDIO_CP_GRID,
  createStarterOristudioCpDocument,
} from './oristudioCpStarterDocument';

describe('Oriedita CP starter document', () => {
  it('starts new CP documents with a square border and readable grid interval', () => {
    const document = createStarterOristudioCpDocument();

    expect(document.crease_pattern.line_segments).toHaveLength(4);
    expect(document.crease_pattern.line_segments.map((line) => line.color)).toEqual([
      'Black0',
      'Black0',
      'Black0',
      'Black0',
    ]);
    expect(document.crease_pattern.line_segments.map((line) => [line.a, line.b])).toEqual([
      [
        { x: ORIEDITA_PAPER_MIN, y: ORIEDITA_PAPER_MAX },
        { x: ORIEDITA_PAPER_MAX, y: ORIEDITA_PAPER_MAX },
      ],
      [
        { x: ORIEDITA_PAPER_MAX, y: ORIEDITA_PAPER_MAX },
        { x: ORIEDITA_PAPER_MAX, y: ORIEDITA_PAPER_MIN },
      ],
      [
        { x: ORIEDITA_PAPER_MAX, y: ORIEDITA_PAPER_MIN },
        { x: ORIEDITA_PAPER_MIN, y: ORIEDITA_PAPER_MIN },
      ],
      [
        { x: ORIEDITA_PAPER_MIN, y: ORIEDITA_PAPER_MIN },
        { x: ORIEDITA_PAPER_MIN, y: ORIEDITA_PAPER_MAX },
      ],
    ]);
    expect(document.crease_pattern.grid).toMatchObject({
      grid_size: 8,
      interval_grid_size: 2,
    });
  });

  it('keeps each starter document independent', () => {
    const first = createStarterOristudioCpDocument();
    const second = createStarterOristudioCpDocument();

    first.crease_pattern.line_segments[0].a.x = 0;
    first.crease_pattern.grid.interval_grid_size = 1;

    expect(second.crease_pattern.line_segments[0].a.x).toBe(ORIEDITA_PAPER_MIN);
    expect(second.crease_pattern.grid).toEqual(STARTER_ORISTUDIO_CP_GRID);
  });
});
