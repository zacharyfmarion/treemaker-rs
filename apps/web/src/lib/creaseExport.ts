import type { CreaseColorMode, CreaseLine, TreeProject } from './sampleProject';

const SIZE = 1024;
const MARGIN = 48;

const FOLD_STYLES: Record<CreaseLine['fold'], string> = {
  mountain: 'stroke:#111417;stroke-width:3',
  valley: 'stroke:#8b2fc6;stroke-width:3;stroke-dasharray:12 8',
  flat: 'stroke:#85919a;stroke-width:1.5',
  border: 'stroke:#111417;stroke-width:4',
};

const KIND_STYLES: Record<CreaseLine['kind'], string> = {
  axial: 'stroke:#111417;stroke-width:3',
  gusset: 'stroke:#737f88;stroke-width:2',
  ridge: 'stroke:#d2545f;stroke-width:2.5',
  hinge: 'stroke:#4d88e8;stroke-width:2.5',
  pseudohinge: 'stroke:#3fbec4;stroke-width:2',
};

function esc(value: string): string {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;');
}

export function serializeCreasePatternSvg(
  project: TreeProject,
  mode: CreaseColorMode = 'mvf'
): string {
  const paperWidth = project.paper.width || 1;
  const paperHeight = project.paper.height || 1;
  const span = SIZE - MARGIN * 2;
  const scale = Math.min(span / paperWidth, span / paperHeight);
  const width = paperWidth * scale;
  const height = paperHeight * scale;
  const offsetX = (SIZE - width) / 2;
  const offsetY = (SIZE - height) / 2;

  const point = (p: { x: number; y: number }) => ({
    x: offsetX + p.x * scale,
    y: offsetY + (paperHeight - p.y) * scale,
  });

  const facets = project.facets
    .map((facet) => {
      const points = facet.vertices
        .map(point)
        .map((p) => `${p.x.toFixed(3)},${p.y.toFixed(3)}`)
        .join(' ');
      const fill =
        facet.color === 'white'
          ? 'rgba(125,183,232,0.18)'
          : facet.color === 'color'
            ? 'rgba(215,168,92,0.18)'
            : 'rgba(95,179,165,0.12)';
      return `  <polygon points="${points}" fill="${fill}" stroke="none"/>`;
    })
    .join('\n');

  const creases = project.creases
    .map((crease) => {
      const a = point(crease.vertices[0]);
      const b = point(crease.vertices[1]);
      const style = mode === 'agrh' ? KIND_STYLES[crease.kind] : FOLD_STYLES[crease.fold];
      return `  <line x1="${a.x.toFixed(3)}" y1="${a.y.toFixed(3)}" x2="${b.x.toFixed(3)}" y2="${b.y.toFixed(3)}" style="${style};fill:none;stroke-linecap:round"/>`;
    })
    .join('\n');

  return [
    '<?xml version="1.0" encoding="UTF-8"?>',
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${SIZE} ${SIZE}" role="img" aria-label="${esc(project.title)} crease pattern">`,
    '  <rect width="100%" height="100%" fill="#ffffff"/>',
    `  <rect x="${offsetX.toFixed(3)}" y="${offsetY.toFixed(3)}" width="${width.toFixed(3)}" height="${height.toFixed(3)}" fill="#f8f5ec" stroke="#111417" stroke-width="3"/>`,
    facets,
    creases,
    `  <rect x="${offsetX.toFixed(3)}" y="${offsetY.toFixed(3)}" width="${width.toFixed(3)}" height="${height.toFixed(3)}" fill="none" stroke="#111417" stroke-width="4"/>`,
    '</svg>',
  ]
    .filter(Boolean)
    .join('\n');
}

export async function renderCreasePatternPng(
  project: TreeProject,
  mode: CreaseColorMode = 'mvf'
): Promise<Uint8Array> {
  const svg = serializeCreasePatternSvg(project, mode);
  const blob = new Blob([svg], { type: 'image/svg+xml;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  try {
    const image = new Image();
    image.decoding = 'async';
    const loaded = new Promise<void>((resolve, reject) => {
      image.onload = () => resolve();
      image.onerror = () => reject(new Error('Failed to render crease pattern SVG'));
    });
    image.src = url;
    await loaded;

    const canvas = document.createElement('canvas');
    canvas.width = SIZE;
    canvas.height = SIZE;
    const ctx = canvas.getContext('2d');
    if (!ctx) throw new Error('Canvas rendering is unavailable');
    ctx.drawImage(image, 0, 0);
    const pngBlob = await new Promise<Blob>((resolve, reject) => {
      canvas.toBlob((result) => {
        if (result) resolve(result);
        else reject(new Error('Failed to encode crease pattern PNG'));
      }, 'image/png');
    });
    return new Uint8Array(await pngBlob.arrayBuffer());
  } finally {
    URL.revokeObjectURL(url);
  }
}
