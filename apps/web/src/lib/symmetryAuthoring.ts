import type { ConditionKind } from '../engine/types';
import type { Point } from './geometry';
import type { TreeProject } from './sampleProject';

export const SYMMETRY_AUTHORING_TOLERANCE = 0.015;

export interface SymmetryAxis {
  loc: Point;
  angle: number;
}

export interface SymmetryLeafPair {
  node1: number;
  node2: number;
  distance: number;
}

export interface SymmetryOnAxisLeaf {
  node: number;
  distance: number;
}

export interface SymmetryAmbiguousLeaf {
  node: number;
  candidates: number[];
}

export interface SymmetryUnmatchedLeaf {
  node: number;
}

export interface SymmetryLeafPreview {
  pairs: SymmetryLeafPair[];
  onAxis: SymmetryOnAxisLeaf[];
  ambiguous: SymmetryAmbiguousLeaf[];
  unmatched: SymmetryUnmatchedLeaf[];
  scopedLeafIds: number[];
}

export interface SymmetryAuthoringPair {
  node1: number;
  node2: number;
}

interface CandidateMatch {
  node: number;
  distance: number;
  candidates: number[];
}

function axisDirection(axis: SymmetryAxis): Point {
  const radians = (axis.angle * Math.PI) / 180;
  return { x: Math.cos(radians), y: Math.sin(radians) };
}

function nodePairKey(node1: number, node2: number): string {
  return node1 < node2 ? `${node1}:${node2}` : `${node2}:${node1}`;
}

function conditionIsPair(kind: ConditionKind, node1: number, node2: number): boolean {
  return (
    kind.type === 'nodes_paired' &&
    ((kind.node1 === node1 && kind.node2 === node2) ||
      (kind.node1 === node2 && kind.node2 === node1))
  );
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

export function hasNodeSymmetricCondition(project: TreeProject, nodeId: number): boolean {
  return project.conditions.some(
    (condition) => condition.kind.type === 'node_symmetric' && condition.kind.node === nodeId
  );
}

export function hasPairedCondition(project: TreeProject, node1: number, node2: number): boolean {
  return project.conditions.some((condition) => conditionIsPair(condition.kind, node1, node2));
}

export function detectSymmetryLeafPairs(
  project: TreeProject,
  nodeIds?: number[],
  tolerance = SYMMETRY_AUTHORING_TOLERANCE
): SymmetryLeafPreview {
  if (!project.hasSymmetry) {
    return { pairs: [], onAxis: [], ambiguous: [], unmatched: [], scopedLeafIds: [] };
  }

  const axis = symmetryAxisForProject(project);
  const selectedIds = nodeIds && nodeIds.length > 0 ? new Set(nodeIds) : null;
  const leaves = project.nodes.filter((node) => node.isLeaf && (!selectedIds || selectedIds.has(node.id)));
  const leafIds = new Set(leaves.map((node) => node.id));
  const scopedLeafIds = leaves.map((node) => node.id);
  const actionableLeaves = leaves.filter(
    (node) => !hasNodeSymmetricCondition(project, node.id) && findPairedNodeId(project, node.id) === null
  );
  const onAxis: SymmetryOnAxisLeaf[] = [];
  const offAxis = actionableLeaves.filter((node) => {
    const distance = distanceToSymmetryAxis(node.loc, axis);
    if (distance <= tolerance) {
      onAxis.push({ node: node.id, distance });
      return false;
    }
    return true;
  });

  const candidates = new Map<number, CandidateMatch>();
  const ambiguous: SymmetryAmbiguousLeaf[] = [];
  const unmatched: SymmetryUnmatchedLeaf[] = [];

  for (const node of offAxis) {
    const reflected = reflectPointAcrossSymmetryAxis(node.loc, axis);
    const side = symmetrySide(node.loc, axis, tolerance);
    const matches = offAxis
      .filter((candidate) => candidate.id !== node.id && leafIds.has(candidate.id))
      .filter((candidate) => symmetrySide(candidate.loc, axis, tolerance) === -side)
      .map((candidate) => ({
        node: candidate.id,
        distance: Math.hypot(candidate.loc.x - reflected.x, candidate.loc.y - reflected.y),
      }))
      .filter((candidate) => candidate.distance <= tolerance)
      .sort((a, b) => a.distance - b.distance || a.node - b.node);

    if (matches.length === 0) {
      unmatched.push({ node: node.id });
      continue;
    }
    if (matches.length > 1) {
      ambiguous.push({ node: node.id, candidates: matches.map((match) => match.node) });
      continue;
    }
    candidates.set(node.id, { ...matches[0], candidates: [matches[0].node] });
  }

  const pairs: SymmetryLeafPair[] = [];
  const pairedKeys = new Set<string>();
  for (const [nodeId, match] of candidates) {
    const reciprocal = candidates.get(match.node);
    if (!reciprocal || reciprocal.node !== nodeId) {
      if (ambiguous.some((item) => item.node === match.node || item.candidates.includes(match.node))) {
        continue;
      }
      unmatched.push({ node: nodeId });
      continue;
    }
    const key = nodePairKey(nodeId, match.node);
    if (pairedKeys.has(key) || hasPairedCondition(project, nodeId, match.node)) continue;
    pairedKeys.add(key);
    pairs.push({
      node1: Math.min(nodeId, match.node),
      node2: Math.max(nodeId, match.node),
      distance: Math.max(match.distance, reciprocal.distance),
    });
  }

  return {
    pairs: pairs.sort((a, b) => a.node1 - b.node1 || a.node2 - b.node2),
    onAxis: onAxis.sort((a, b) => a.node - b.node),
    ambiguous: ambiguous.sort((a, b) => a.node - b.node),
    unmatched: unmatched.sort((a, b) => a.node - b.node),
    scopedLeafIds,
  };
}
