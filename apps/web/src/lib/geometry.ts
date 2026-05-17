export interface Point {
  x: number;
  y: number;
}

export interface PlotRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export function paperToSvg(point: Point, rect: PlotRect): Point {
  return {
    x: rect.x + point.x * rect.width,
    y: rect.y + (1 - point.y) * rect.height,
  };
}

export function svgToPaper(point: Point, rect: PlotRect): Point {
  return {
    x: (point.x - rect.x) / rect.width,
    y: 1 - (point.y - rect.y) / rect.height,
  };
}

export function clampPaperPoint(point: Point): Point {
  return {
    x: Math.min(1, Math.max(0, point.x)),
    y: Math.min(1, Math.max(0, point.y)),
  };
}

export function formatNumber(value: number, digits = 3): string {
  return value.toFixed(digits).replace(/\.?0+$/, '');
}
