import type { Point } from '../lib/geometry';

export interface TreeSnapshot {
  summary: TreeSummary;
  cp_status_report: CpStatusReport;
  paper: PaperSettings;
  nodes: NodeSnapshot[];
  edges: EdgeSnapshot[];
  paths: PathSnapshot[];
  vertices: VertexSnapshot[];
  creases: CreaseSnapshot[];
  facets: FacetSnapshot[];
}

export interface TreeSummary {
  scale: number;
  is_feasible: boolean;
  cp_status: string;
  nodes: number;
  edges: number;
  paths: number;
  vertices: number;
  creases: number;
  facets: number;
  leaf_nodes: number;
}

export interface CpStatusReport {
  status: string;
  bad_edges: number[];
  bad_polys: number[];
  bad_vertices: number[];
  bad_creases: number[];
  bad_facets: number[];
}

export interface PaperSettings {
  width: number;
  height: number;
  scale: number;
  has_symmetry: boolean;
  sym_loc: Point;
  sym_angle: number;
}

export interface NodeSnapshot {
  id: number;
  label: string;
  loc: Point;
  is_leaf: boolean;
  is_pinned: boolean;
  owner: unknown;
}

export interface EdgeSnapshot {
  id: number;
  label: string;
  nodes: number[];
  length: number;
  strain: number;
  stiffness: number;
}

export interface PathSnapshot {
  id: number;
  nodes: number[];
  is_leaf: boolean;
  is_active: boolean;
  is_feasible: boolean;
  is_border: boolean;
}

export interface VertexSnapshot {
  id: number;
  loc: Point;
}

export interface CreaseSnapshot {
  id: number;
  kind: number;
  vertices: number[];
  fold: number;
}

export interface FacetSnapshot {
  id: number;
  vertices: number[];
  color: number;
}

export type TreeEdit =
  | {
      type: 'add_node';
      loc: Point;
      label?: string;
      connect_to?: number;
      edge_length?: number;
    }
  | { type: 'move_node'; id: number; loc: Point }
  | { type: 'delete_node'; id: number }
  | { type: 'update_node_label'; id: number; label: string }
  | {
      type: 'update_edge';
      id: number;
      label?: string;
      length?: number;
      strain?: number;
      stiffness?: number;
    }
  | { type: 'add_edge'; node1: number; node2: number; label?: string; length?: number }
  | { type: 'delete_edge'; id: number };

export interface EditReport {
  snapshot: TreeSnapshot;
  created_node?: number;
  created_edge?: number;
}

export interface OptimizationReport {
  kind: string;
  converged: boolean;
  old_scale: number;
  new_scale: number;
  is_feasible: boolean;
  message: string;
}

export interface WasmErrorEnvelope {
  code: string;
  message: string;
}
