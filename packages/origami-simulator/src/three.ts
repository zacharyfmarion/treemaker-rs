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
      geometry.setAttribute('position', new THREE.BufferAttribute(frame.positions.slice(), 3));
      geometry.setAttribute('color', new THREE.BufferAttribute(frame.colors.slice(), 3));
      geometry.computeVertexNormals();
      lineGeometry.setAttribute('position', new THREE.BufferAttribute(edgePositions(model, frame.positions), 3));
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
    out.set(positions.slice(edge[0] * 3, edge[0] * 3 + 3), edgeIndex * 6);
    out.set(positions.slice(edge[1] * 3, edge[1] * 3 + 3), edgeIndex * 6 + 3);
  });
  return out;
}
