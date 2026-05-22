import type { Point } from '../lib/geometry';
import type {
  OristudioCpOperationId,
  OristudioCpOperationStatus,
} from '../lib/oristudioCpCommands';

export type OristudioCpOperationCategory =
  | 'Kernel'
  | 'Io'
  | 'KernelIntent'
  | 'KernelPreview'
  | 'UiPreviewOnly'
  | 'OutOfScopeUi';

export interface OristudioCpOperationDescriptor {
  id: OristudioCpOperationId;
  upstream: string;
  target: string;
  category: OristudioCpOperationCategory;
  stage: number;
  status: OristudioCpOperationStatus;
}

export interface OristudioCpRgbColor {
  red: number;
  green: number;
  blue: number;
}

export interface OristudioCpLineSegment {
  a: Point;
  b: Point;
  active: string;
  color: string;
  selected: number;
  customized: number;
  customized_color: OristudioCpRgbColor;
}

export interface OristudioCpCircle {
  x: number;
  y: number;
  r: number;
  color: string;
  customized: number;
  customized_color: OristudioCpRgbColor;
}

export interface OristudioCpTextElement {
  x: number | { 0: number };
  y: number | { 0: number };
  text: string;
}

export interface OristudioCpGridMetadata {
  interval_grid_size: number;
  grid_size: number;
  grid_xa: number;
  grid_xb: number;
  grid_xc: number;
  grid_ya: number;
  grid_yb: number;
  grid_yc: number;
  grid_angle: number;
  base_state: string;
  vertical_scale_position: number;
  horizontal_scale_position: number;
  draw_diagonal_gridlines: boolean;
}

export interface OristudioCpModel {
  line_segments: OristudioCpLineSegment[];
  circles: OristudioCpCircle[];
  points: Point[];
  aux_line_segments: OristudioCpLineSegment[];
  texts: OristudioCpTextElement[];
  grid: OristudioCpGridMetadata;
}

export interface OristudioCpDocumentSnapshot {
  title?: string | null;
  crease_pattern: OristudioCpModel;
  metadata: Record<string, unknown>;
}

export interface OristudioCpDocumentSummary {
  title?: string | null;
  line_segments: number;
  circles: number;
  points: number;
  aux_line_segments: number;
  texts: number;
  can_save_as_cp: boolean;
  is_empty: boolean;
}

export interface OristudioCpCommandResult {
  operation: OristudioCpOperationId;
  status: OristudioCpOperationStatus;
  diagnostics: string[];
}

export interface OristudioCpCommandPayload {
  line_ids?: number[];
}

export interface OristudioCpDocumentState {
  handle: number;
  document: OristudioCpDocumentSnapshot;
  summary: OristudioCpDocumentSummary;
  source: {
    format: 'cp' | 'fold';
    filename: string;
    path: string | null;
  };
  operationDescriptors: OristudioCpOperationDescriptor[];
  lastCommandResult: OristudioCpCommandResult | null;
}
