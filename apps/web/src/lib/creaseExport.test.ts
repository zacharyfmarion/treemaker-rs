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

  it('uses fold colors for M/V mode and structural colors for AGRH mode', () => {
    const project = createSampleProject();

    const mvf = serializeCreasePatternSvg(project, { viewMode: 'mvf', includeUnassigned: true });
    const agrh = serializeCreasePatternSvg(project, { viewMode: 'agrh', includeUnassigned: true });

    expect(mvf).toContain('stroke:#ff4d5d;stroke-width:3');
    expect(mvf).toContain('stroke:#60a5fa;stroke-width:3;stroke-dasharray:10 7');
    expect(mvf).not.toContain('stroke:#d2545f');
    expect(agrh).toContain('stroke:#d2545f');
    expect(agrh).not.toContain('stroke:#60a5fa');
  });

  it('can hide flat and unassigned creases', () => {
    const svg = serializeCreasePatternSvg(createSampleProject(), {
      viewMode: 'mvf',
      includeUnassigned: false,
    });

    expect(svg.match(/<line /g)).toHaveLength(4);
    expect(svg).not.toContain('stroke:#85919a');
  });

  it('can hide background facet colors', () => {
    const svg = serializeCreasePatternSvg(createSampleProject(), {
      viewMode: 'mvf',
      includeUnassigned: true,
      showBackgroundColor: false,
    });

    expect(svg).toContain('fill="#ffffff" stroke="#111417" stroke-width="3"');
    expect(svg).not.toContain('<polygon');
    expect(svg).not.toContain('rgba(125,183,232,0.18)');
    expect(svg).not.toContain('rgba(215,168,92,0.18)');
    expect(svg).not.toContain('rgba(95,179,165,0.12)');
    expect(svg).toContain('<line');
  });
});
