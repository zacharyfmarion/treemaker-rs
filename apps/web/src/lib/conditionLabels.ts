import type { ConditionKind } from '../engine/types';

function pathText(node1: number, node2: number): string {
  return `Path ${node1} -> ${node2}`;
}

export function conditionTitle(kind: ConditionKind): string {
  switch (kind.type) {
    case 'node_combo':
      return `Node ${kind.node} combined`;
    case 'node_fixed':
      return `Node ${kind.node} fixed`;
    case 'node_on_corner':
      return `Node ${kind.node} on corner`;
    case 'node_on_edge':
      return `Node ${kind.node} on edge`;
    case 'node_symmetric':
      return `Node ${kind.node} on symmetry`;
    case 'nodes_paired':
      return `Nodes ${kind.node1} and ${kind.node2} paired`;
    case 'nodes_collinear':
      return `Nodes ${kind.node1}, ${kind.node2}, ${kind.node3} collinear`;
    case 'edge_length_fixed':
      return `Edge ${kind.edge} length fixed`;
    case 'edges_same_strain':
      return `Edges ${kind.edge1} and ${kind.edge2} same strain`;
    case 'path_combo':
      return `${pathText(kind.node1, kind.node2)} combined`;
    case 'path_active':
      return `${pathText(kind.node1, kind.node2)} active`;
    case 'path_angle_fixed':
      return `${pathText(kind.node1, kind.node2)} angle fixed`;
    case 'path_angle_quant':
      return `${pathText(kind.node1, kind.node2)} angle quantized`;
  }
}

export function conditionDetail(kind: ConditionKind): string {
  switch (kind.type) {
    case 'node_combo':
      return [
        kind.to_symmetry_line ? 'symmetry line' : '',
        kind.to_paper_edge ? 'paper edge' : '',
        kind.to_paper_corner ? 'paper corner' : '',
        kind.x_fixed ? `x=${kind.x_fix_value.toFixed(3)}` : '',
        kind.y_fixed ? `y=${kind.y_fix_value.toFixed(3)}` : '',
      ]
        .filter(Boolean)
        .join(', ');
    case 'node_fixed':
      return [
        kind.x_fixed ? `x=${kind.x_fix_value.toFixed(3)}` : '',
        kind.y_fixed ? `y=${kind.y_fix_value.toFixed(3)}` : '',
      ]
        .filter(Boolean)
        .join(', ');
    case 'path_angle_fixed':
      return `${kind.angle.toFixed(2)} deg`;
    case 'path_angle_quant':
      return `${kind.quant} divisions, offset ${kind.quant_offset.toFixed(2)} deg`;
    case 'path_combo':
      return [
        kind.is_angle_fixed ? `${kind.angle.toFixed(2)} deg` : '',
        kind.is_angle_quant ? `${kind.quant} divisions` : '',
      ]
        .filter(Boolean)
        .join(', ');
    default:
      return kind.type.replaceAll('_', ' ');
  }
}
