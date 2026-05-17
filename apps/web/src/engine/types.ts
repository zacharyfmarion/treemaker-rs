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
  conditions: ConditionSnapshot[];
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
  conditions: number;
  conditioned_nodes: number;
  conditioned_edges: number;
  conditioned_paths: number;
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
  is_conditioned: boolean;
  owner: unknown;
}

export interface EdgeSnapshot {
  id: number;
  label: string;
  nodes: number[];
  length: number;
  strain: number;
  stiffness: number;
  is_conditioned: boolean;
}

export interface PathSnapshot {
  id: number;
  nodes: number[];
  is_leaf: boolean;
  is_active: boolean;
  is_feasible: boolean;
  is_border: boolean;
  is_conditioned: boolean;
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

export interface ConditionSnapshot {
  index: number;
  is_feasible: boolean;
  kind: ConditionKind;
}

export type ConditionKind =
  | {
      type: 'node_combo';
      node: number;
      to_symmetry_line: boolean;
      to_paper_edge: boolean;
      to_paper_corner: boolean;
      x_fixed: boolean;
      x_fix_value: number;
      y_fixed: boolean;
      y_fix_value: number;
    }
  | {
      type: 'node_fixed';
      node: number;
      x_fixed: boolean;
      y_fixed: boolean;
      x_fix_value: number;
      y_fix_value: number;
    }
  | { type: 'node_on_corner'; node: number }
  | { type: 'node_on_edge'; node: number }
  | { type: 'node_symmetric'; node: number }
  | { type: 'nodes_paired'; node1: number; node2: number }
  | { type: 'nodes_collinear'; node1: number; node2: number; node3: number }
  | { type: 'edge_length_fixed'; edge: number }
  | { type: 'edges_same_strain'; edge1: number; edge2: number }
  | {
      type: 'path_combo';
      node1: number;
      node2: number;
      is_angle_fixed: boolean;
      angle: number;
      is_angle_quant: boolean;
      quant: number;
      quant_offset: number;
    }
  | { type: 'path_active'; node1: number; node2: number }
  | { type: 'path_angle_fixed'; node1: number; node2: number; angle: number }
  | { type: 'path_angle_quant'; node1: number; node2: number; quant: number; quant_offset: number };

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
  | { type: 'delete_edge'; id: number }
  | { type: 'update_paper'; width: number; height: number; scale?: number }
  | {
      type: 'set_symmetry';
      has_symmetry: boolean;
      sym_loc?: Point;
      sym_angle?: number;
    }
  | { type: 'add_condition'; kind: ConditionKind }
  | { type: 'update_condition'; id: number; kind: ConditionKind }
  | { type: 'delete_condition'; id: number };

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
