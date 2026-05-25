import type { OristudioCpDiagnosticEntry } from '../engine/oristudioCpTypes';

export function semanticCpDiagnosticKind(kind: string): string {
  switch (kind) {
    case 'Check1':
      return 'Overlap check';
    case 'Check2':
      return 'T-junction check';
    case 'Check3':
      return 'Vertex foldability';
    case 'Check4':
    case 'CheckCamv':
      return 'Maekawa/LBL';
    default:
      return kind;
  }
}

export function semanticCpDiagnosticSummary(summary: string | null): string | null {
  if (!summary) return null;
  return summary
    .replace(/^Check1\b/u, 'Overlap check')
    .replace(/^Check2\b/u, 'T-junction check')
    .replace(/^Check3\b/u, 'Vertex foldability check')
    .replace(/^Check4\b/u, 'Maekawa/LBL check')
    .replace(/^CheckCamv\b/u, 'CAMV check');
}

export function cpDiagnosticEntryMessage(entry: OristudioCpDiagnosticEntry): string {
  return entry.message;
}
