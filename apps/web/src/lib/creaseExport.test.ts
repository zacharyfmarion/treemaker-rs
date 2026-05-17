import { describe, expect, it } from 'vitest';
import { createSampleProject } from './sampleProject';
import { serializeCreasePatternSvg } from './creaseExport';

describe('crease pattern export', () => {
  it('serializes creases and facets to SVG', () => {
    const svg = serializeCreasePatternSvg(createSampleProject());

    expect(svg).toContain('<svg');
    expect(svg).toContain('<polygon');
    expect(svg).toContain('<line');
    expect(svg).toContain('stroke-dasharray');
  });
});
