import { describe, expect, it } from 'vitest';
import { clampPaperPoint, paperToSvg, svgToPaper } from './geometry';

describe('paper coordinate transforms', () => {
  const rect = { x: 10, y: 20, width: 200, height: 100 };

  it('maps paper coordinates to SVG coordinates with inverted y', () => {
    expect(paperToSvg({ x: 0, y: 0 }, rect)).toEqual({ x: 10, y: 120 });
    expect(paperToSvg({ x: 1, y: 1 }, rect)).toEqual({ x: 210, y: 20 });
    expect(paperToSvg({ x: 0.5, y: 0.25 }, rect)).toEqual({ x: 110, y: 95 });
  });

  it('round trips through SVG coordinates', () => {
    const point = { x: 0.27, y: 0.82 };
    const svg = paperToSvg(point, rect);
    expect(svgToPaper(svg, rect).x).toBeCloseTo(point.x);
    expect(svgToPaper(svg, rect).y).toBeCloseTo(point.y);
  });

  it('clamps points to paper bounds', () => {
    expect(clampPaperPoint({ x: -0.5, y: 1.5 })).toEqual({ x: 0, y: 1 });
  });
});
