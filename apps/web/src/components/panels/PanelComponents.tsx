import type { FC } from 'react';
import type { IDockviewPanelProps } from 'dockview';
import { DesignPanel } from './DesignPanel';
import { InspectorPanel } from './InspectorPanel';
import { CreasePatternPanel } from './CreasePatternPanel';
import { DiagnosticsPanel } from './DiagnosticsPanel';
import { FilesPanel } from './FilesPanel';
import { ConditionsPanel } from './ConditionsPanel';

export const panelComponents: Record<string, FC<IDockviewPanelProps>> = {
  design: DesignPanel,
  inspector: InspectorPanel,
  'crease-pattern': CreasePatternPanel,
  diagnostics: DiagnosticsPanel,
  files: FilesPanel,
  conditions: ConditionsPanel,
};
