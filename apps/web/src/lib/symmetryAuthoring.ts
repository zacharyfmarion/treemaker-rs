import type { Point } from './geometry';
import type { TreeProject } from './sampleProject';

export const SYMMETRY_AUTHORING_TOLERANCE = 0.015;

export interface SymmetryAxis {
  loc: Point;
  angle: number;
}

export interface SymmetryAuthoringPair {
  node1: number;
  node2: number;
}

function axisDirection(axis: SymmetryAxis): Point {
  const radians = (axis.angle * Math.PI) / 180;
  return { x: Math.cos(radians), y: Math.sin(radians) };
}

function nodeExists(project: TreeProject, nodeId: number): boolean {
  return project.nodes.some((node) => node.id === nodeId);
}

export function symmetryAxisForProject(project: TreeProject): SymmetryAxis {
  return {
    loc: project.paper.symLoc,
    angle: project.paper.symAngle,
  };
}

export function projectOntoSymmetryAxis(point: Point, axis: SymmetryAxis): Point {
  const direction = axisDirection(axis);
  const dx = point.x - axis.loc.x;
  const dy = point.y - axis.loc.y;
  const dot = dx * direction.x + dy * direction.y;
  return {
    x: axis.loc.x + dot * direction.x,
    y: axis.loc.y + dot * direction.y,
  };
}

export function reflectPointAcrossSymmetryAxis(point: Point, axis: SymmetryAxis): Point {
  const projected = projectOntoSymmetryAxis(point, axis);
  return {
    x: 2 * projected.x - point.x,
    y: 2 * projected.y - point.y,
  };
}

export function distanceToSymmetryAxis(point: Point, axis: SymmetryAxis): number {
  const direction = axisDirection(axis);
  const dx = point.x - axis.loc.x;
  const dy = point.y - axis.loc.y;
  return Math.abs(dx * direction.y - dy * direction.x);
}

export function symmetrySide(point: Point, axis: SymmetryAxis, tolerance = SYMMETRY_AUTHORING_TOLERANCE): -1 | 0 | 1 {
  const direction = axisDirection(axis);
  const dx = point.x - axis.loc.x;
  const dy = point.y - axis.loc.y;
  const cross = dx * direction.y - dy * direction.x;
  if (Math.abs(cross) <= tolerance) return 0;
  return cross < 0 ? -1 : 1;
}

export function snapPointToSymmetryAxis(
  point: Point,
  axis: SymmetryAxis,
  tolerance = SYMMETRY_AUTHORING_TOLERANCE
): { point: Point; snapped: boolean; distance: number } {
  const distance = distanceToSymmetryAxis(point, axis);
  if (distance > tolerance) return { point, snapped: false, distance };
  return { point: projectOntoSymmetryAxis(point, axis), snapped: true, distance };
}

export function findPairedNodeId(project: TreeProject, nodeId: number): number | null {
  for (const condition of project.conditions) {
    if (condition.kind.type !== 'nodes_paired') continue;
    if (condition.kind.node1 === nodeId) return condition.kind.node2;
    if (condition.kind.node2 === nodeId) return condition.kind.node1;
  }
  return null;
}

export function addSymmetryAuthoringPair(
  pairs: SymmetryAuthoringPair[],
  node1: number,
  node2: number
): SymmetryAuthoringPair[] {
  if (node1 === node2) return pairs;
  const nextPair = {
    node1: Math.min(node1, node2),
    node2: Math.max(node1, node2),
  };
  if (pairs.some((pair) => pair.node1 === nextPair.node1 && pair.node2 === nextPair.node2)) {
    return pairs;
  }
  return [...pairs, nextPair];
}

export function filterSymmetryAuthoringPairs(
  project: TreeProject,
  pairs: SymmetryAuthoringPair[]
): SymmetryAuthoringPair[] {
  return pairs.filter(
    (pair) =>
      pair.node1 !== pair.node2 &&
      nodeExists(project, pair.node1) &&
      nodeExists(project, pair.node2)
  );
}

export function findSymmetryAuthoringPairId(
  project: TreeProject,
  pairs: SymmetryAuthoringPair[],
  nodeId: number
): number | null {
  const pair = pairs.find(
    (candidate) =>
      (candidate.node1 === nodeId || candidate.node2 === nodeId) &&
      nodeExists(project, candidate.node1) &&
      nodeExists(project, candidate.node2)
  );
  if (!pair) return null;
  return pair.node1 === nodeId ? pair.node2 : pair.node1;
}

export function findMirrorNodeId(
  project: TreeProject,
  pairs: SymmetryAuthoringPair[],
  nodeId: number
): number | null {
  return findPairedNodeId(project, nodeId) ?? findSymmetryAuthoringPairId(project, pairs, nodeId);
}

function mirroredNodeForEdgeEndpoint(
  project: TreeProject,
  pairs: SymmetryAuthoringPair[],
  nodeId: number
): number | null {
  const paired = findMirrorNodeId(project, pairs, nodeId);
  if (paired) return paired;
  const node = project.nodes.find((candidate) => candidate.id === nodeId);
  if (!node || !project.hasSymmetry) return null;
  return symmetrySide(node.loc, symmetryAxisForProject(project)) === 0 ? node.id : null;
}

export function findMirrorEdgeId(
  project: TreeProject,
  pairs: SymmetryAuthoringPair[],
  edgeId: number
): number | null {
  const edge = project.edges.find((candidate) => candidate.id === edgeId);
  if (!edge || !project.hasSymmetry) return null;
  const node1 = mirroredNodeForEdgeEndpoint(project, pairs, edge.nodes[0]);
  const node2 = mirroredNodeForEdgeEndpoint(project, pairs, edge.nodes[1]);
  if (!node1 || !node2 || (node1 === edge.nodes[0] && node2 === edge.nodes[1])) return null;
  const mirrored = project.edges.find(
    (candidate) =>
      candidate.id !== edge.id &&
      ((candidate.nodes[0] === node1 && candidate.nodes[1] === node2) ||
        (candidate.nodes[0] === node2 && candidate.nodes[1] === node1))
  );
  return mirrored?.id ?? null;
}
