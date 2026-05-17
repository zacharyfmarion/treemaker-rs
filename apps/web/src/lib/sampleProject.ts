import type { Point } from './geometry';

export type Selection =
  | { kind: 'tree' }
  | { kind: 'node'; id: number }
  | { kind: 'edge'; id: number }
  | { kind: 'crease'; id: number };

export type AppStatus =
  | 'loading_engine'
  | 'ready'
  | 'needs_optimization'
  | 'optimizing'
  | 'optimized'
  | 'building_crease_pattern'
  | 'crease_pattern_ready'
  | 'error';
export type ToolMode = 'select' | 'node' | 'edge';
export type CreaseColorMode = 'mvf' | 'agrh';

export interface TreeNode {
  id: number;
  label: string;
  loc: Point;
  isLeaf: boolean;
  isPinned: boolean;
}

export interface TreeEdge {
  id: number;
  label: string;
  nodes: [number, number];
  length: number;
  strain: number;
  stiffness: number;
}

export interface TreePath {
  id: number;
  nodes: [number, number];
  isActive: boolean;
  isFeasible: boolean;
  isBorder: boolean;
}

export interface CreaseLine {
  id: number;
  vertices: [Point, Point];
  fold: 'mountain' | 'valley' | 'flat' | 'border';
  kind: 'axial' | 'gusset' | 'ridge' | 'hinge' | 'pseudohinge';
}

export interface FacetShape {
  id: number;
  vertices: Point[];
  color: 'white' | 'color' | 'unset';
}

export interface TreeProject {
  title: string;
  paper: { width: number; height: number };
  scale: number;
  hasSymmetry: boolean;
  nodes: TreeNode[];
  edges: TreeEdge[];
  paths: TreePath[];
  creases: CreaseLine[];
  facets: FacetShape[];
}

export function createEmptyProject(): TreeProject {
  return {
    title: 'Editable tree design',
    paper: { width: 1, height: 1 },
    scale: 0.1,
    hasSymmetry: false,
    nodes: [],
    edges: [],
    paths: [],
    creases: [],
    facets: [],
  };
}

export function createSampleProject(): TreeProject {
  return {
    title: 'Untitled tree',
    paper: { width: 1, height: 1 },
    scale: 0.1,
    hasSymmetry: false,
    nodes: [
      { id: 1, label: 'root', loc: { x: 0.5, y: 0.46 }, isLeaf: false, isPinned: false },
      { id: 2, label: 't0', loc: { x: 0.2, y: 0.2 }, isLeaf: true, isPinned: false },
      { id: 3, label: 't1', loc: { x: 0.82, y: 0.22 }, isLeaf: true, isPinned: false },
      { id: 4, label: 't2', loc: { x: 0.5, y: 0.82 }, isLeaf: true, isPinned: false },
    ],
    edges: [
      { id: 1, label: 'e1', nodes: [1, 2], length: 1, strain: 0, stiffness: 1 },
      { id: 2, label: 'e2', nodes: [1, 3], length: 1, strain: 0, stiffness: 1 },
      { id: 3, label: 'e3', nodes: [1, 4], length: 1, strain: 0, stiffness: 1 },
    ],
    paths: [
      { id: 1, nodes: [2, 3], isActive: true, isFeasible: true, isBorder: true },
      { id: 2, nodes: [3, 4], isActive: true, isFeasible: true, isBorder: true },
      { id: 3, nodes: [2, 4], isActive: true, isFeasible: true, isBorder: true },
    ],
    facets: [
      {
        id: 1,
        color: 'white',
        vertices: [
          { x: 0.14, y: 0.14 },
          { x: 0.86, y: 0.14 },
          { x: 0.5, y: 0.5 },
        ],
      },
      {
        id: 2,
        color: 'color',
        vertices: [
          { x: 0.86, y: 0.14 },
          { x: 0.82, y: 0.88 },
          { x: 0.5, y: 0.5 },
        ],
      },
      {
        id: 3,
        color: 'unset',
        vertices: [
          { x: 0.82, y: 0.88 },
          { x: 0.14, y: 0.86 },
          { x: 0.5, y: 0.5 },
        ],
      },
    ],
    creases: [
      {
        id: 1,
        vertices: [
          { x: 0.14, y: 0.14 },
          { x: 0.86, y: 0.14 },
        ],
        fold: 'border',
        kind: 'axial',
      },
      {
        id: 2,
        vertices: [
          { x: 0.86, y: 0.14 },
          { x: 0.82, y: 0.88 },
        ],
        fold: 'mountain',
        kind: 'ridge',
      },
      {
        id: 3,
        vertices: [
          { x: 0.82, y: 0.88 },
          { x: 0.14, y: 0.86 },
        ],
        fold: 'valley',
        kind: 'hinge',
      },
      {
        id: 4,
        vertices: [
          { x: 0.14, y: 0.86 },
          { x: 0.14, y: 0.14 },
        ],
        fold: 'border',
        kind: 'axial',
      },
      {
        id: 5,
        vertices: [
          { x: 0.14, y: 0.14 },
          { x: 0.5, y: 0.5 },
        ],
        fold: 'flat',
        kind: 'gusset',
      },
      {
        id: 6,
        vertices: [
          { x: 0.5, y: 0.5 },
          { x: 0.82, y: 0.88 },
        ],
        fold: 'flat',
        kind: 'pseudohinge',
      },
    ],
  };
}
