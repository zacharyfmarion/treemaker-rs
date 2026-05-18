import { describe, expect, it } from 'vitest';
import { createSampleProject } from './sampleProject';
import { paperToSvg } from './geometry';
import {
  DEFAULT_DESIGN_VIEW_LAYERS,
  DESIGN_PAPER_RECT,
  clientPointToPaper,
  getDesignWorldRect,
  setDesignLayerVisibility,
} from './designViewport';

describe('design viewport helpers', () => {
  it('expands world bounds for large visible leaf circles outside the paper', () => {
    const project = createSampleProject();
    project.scale = 1;

    const rect = getDesignWorldRect(project, DEFAULT_DESIGN_VIEW_LAYERS);

    expect(rect.x).toBeLessThan(0);
    expect(rect.y).toBeLessThan(0);
    expect(rect.x + rect.width).toBeGreaterThan(720);
    expect(rect.y + rect.height).toBeGreaterThan(720);
  });

  it('maps client coordinates to paper coordinates with a nonzero world viewBox', () => {
    const targetPaperPoint = { x: 0.25, y: 0.75 };
    const targetWorldPoint = paperToSvg(targetPaperPoint, DESIGN_PAPER_RECT);
    const worldRect = { x: -200, y: -100, width: 1000, height: 900 };
    const bounds = { left: 10, top: 20, width: 500, height: 450 };
    const client = {
      x: bounds.left + ((targetWorldPoint.x - worldRect.x) / worldRect.width) * bounds.width,
      y: bounds.top + ((targetWorldPoint.y - worldRect.y) / worldRect.height) * bounds.height,
    };

    const paperPoint = clientPointToPaper(client, bounds, worldRect);

    expect(paperPoint.x).toBeCloseTo(targetPaperPoint.x);
    expect(paperPoint.y).toBeCloseTo(targetPaperPoint.y);
  });

  it('keeps layer visibility state separate from project data', () => {
    const project = createSampleProject();
    const before = JSON.stringify(project);

    const layers = setDesignLayerVisibility(DEFAULT_DESIGN_VIEW_LAYERS, 'labels', false);
    getDesignWorldRect(project, layers);

    expect(layers).not.toBe(DEFAULT_DESIGN_VIEW_LAYERS);
    expect(layers.labels).toBe(false);
    expect(DEFAULT_DESIGN_VIEW_LAYERS.labels).toBe(true);
    expect(JSON.stringify(project)).toBe(before);
  });
});
