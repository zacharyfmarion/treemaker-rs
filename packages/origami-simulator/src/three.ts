import * as THREE from 'three';
import type { PreparedOrigamiModel, SimulationFrame } from './types.js';

export interface ThreeOrigamiRenderer {
  group: THREE.Group;
  mesh: THREE.Mesh;
  edgeLines: THREE.LineSegments;
  update(frame: SimulationFrame): void;
  dispose(): void;
}

export function createThreeOrigamiRenderer(model: PreparedOrigamiModel): ThreeOrigamiRenderer {
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.BufferAttribute(model.positions.slice(), 3));
  geometry.setAttribute('color', new THREE.BufferAttribute(model.colors.slice(), 3));
  geometry.setIndex(new THREE.BufferAttribute(model.indices, 1));
  geometry.computeVertexNormals();

  const material = new THREE.MeshPhongMaterial({
    side: THREE.DoubleSide,
    flatShading: true,
    vertexColors: true,
  });
  const mesh = new THREE.Mesh(geometry, material);

  const lineGeometry = new THREE.BufferGeometry();
  lineGeometry.setAttribute('position', new THREE.BufferAttribute(edgePositions(model), 3));
  const lineMaterial = new THREE.LineBasicMaterial({ color: 0x202020 });
  const edgeLines = new THREE.LineSegments(lineGeometry, lineMaterial);

  const group = new THREE.Group();
  group.add(mesh);
  group.add(edgeLines);

  return {
    group,
    mesh,
    edgeLines,
    update(frame: SimulationFrame) {
      const position = geometry.getAttribute('position') as THREE.BufferAttribute;
      const color = geometry.getAttribute('color') as THREE.BufferAttribute;
      position.array.set(frame.positions);
      color.array.set(frame.colors);
      position.needsUpdate = true;
      color.needsUpdate = true;
      geometry.computeVertexNormals();
      const linePosition = lineGeometry.getAttribute('position') as THREE.BufferAttribute;
      linePosition.array.set(edgePositions(model, frame.positions));
      linePosition.needsUpdate = true;
    },
    dispose() {
      geometry.dispose();
      material.dispose();
      lineGeometry.dispose();
      lineMaterial.dispose();
    },
  };
}

function edgePositions(model: PreparedOrigamiModel, positions = model.positions): Float32Array {
  const out = new Float32Array(model.edgesVertices.length * 2 * 3);
  model.edgesVertices.forEach((edge, edgeIndex) => {
    const sourceA = edge[0] * 3;
    const sourceB = edge[1] * 3;
    const target = edgeIndex * 6;
    out[target] = positions[sourceA] ?? 0;
    out[target + 1] = positions[sourceA + 1] ?? 0;
    out[target + 2] = positions[sourceA + 2] ?? 0;
    out[target + 3] = positions[sourceB] ?? 0;
    out[target + 4] = positions[sourceB + 1] ?? 0;
    out[target + 5] = positions[sourceB + 2] ?? 0;
  });
  return out;
}
