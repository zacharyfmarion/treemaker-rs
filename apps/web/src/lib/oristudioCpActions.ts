import type { OristudioCpLineColor } from '../engine/oristudioCpTypes';
import {
  ORISTUDIO_CP_COMMAND_GROUPS,
  ORISTUDIO_CP_COMMANDS,
  cpCommandByOperation,
  type OristudioCpCommandDefinition,
  type OristudioCpCommandGroupId,
  type OristudioCpCommandPlacement,
  type OristudioCpCommandUiStatus,
  type OristudioCpOperationId,
} from './oristudioCpCommands';

export type OristudioCpActionKind = 'line-type' | 'command';
export type OristudioCpActionId = `cp.action.${string}`;
export type OristudioCpActionGroupId = 'line-type' | OristudioCpCommandGroupId;
export type OristudioCpLineInputMode = 'fold-line' | 'aux-line';
export type OristudioCpActionInputMode = 'point-sequence' | 'drag-path' | 'drag-line';

export interface OristudioCpActionGroupDefinition {
  id: OristudioCpActionGroupId;
  label: string;
  railLabel: string;
  order: number;
}

export interface OristudioCpBaseActionDefinition {
  id: OristudioCpActionId;
  kind: OristudioCpActionKind;
  label: string;
  railLabel?: string;
  group: OristudioCpActionGroupId;
  placement: OristudioCpCommandPlacement;
  icon: string;
  upstreamAction: string;
  upstreamMouseMode?: string;
  tooltip: string;
  uiStatus: OristudioCpCommandUiStatus;
  disabledReason: string;
  shortcut?: string;
}

export interface OristudioCpLineTypeActionDefinition
  extends OristudioCpBaseActionDefinition {
  kind: 'line-type';
  group: 'line-type';
  lineColor: OristudioCpLineColor;
  lineInputMode: OristudioCpLineInputMode;
}

export interface OristudioCpCommandActionDefinition
  extends OristudioCpBaseActionDefinition {
  kind: 'command';
  command: OristudioCpCommandDefinition;
  operationId: OristudioCpOperationId;
  toolSteps?: readonly string[];
  inputMode?: OristudioCpActionInputMode;
  lineInputMode?: OristudioCpLineInputMode;
  repeatable?: boolean;
}

export type OristudioCpActionDefinition =
  | OristudioCpLineTypeActionDefinition
  | OristudioCpCommandActionDefinition;

export const ORISTUDIO_CP_LINE_TYPE_ACTIONS = [
  lineTypeAction('mountain', 'Mountain', 'M', 'Red1', 'colRedAction', 'line-type-mountain'),
  lineTypeAction('valley', 'Valley', 'V', 'Blue2', 'colBlueAction', 'line-type-valley'),
  lineTypeAction('edge', 'Edge', 'E', 'Black0', 'colBlackAction', 'line-type-edge'),
  lineTypeAction('auxiliary', 'Auxiliary', 'A', 'Cyan3', 'colCyanAction', 'line-type-auxiliary'),
] as const satisfies readonly OristudioCpLineTypeActionDefinition[];

const AUXILIARY_DRAW_COMMAND = cpCommandByOperation('DrawCreaseFree');

export const ORISTUDIO_CP_ACTION_GROUPS: OristudioCpActionGroupDefinition[] = [
  { id: 'line-type', label: 'Line type', railLabel: 'Type', order: 5 },
  ...ORISTUDIO_CP_COMMAND_GROUPS.map((group) => ({
    id: group.id,
    label: group.label,
    railLabel: group.railLabel,
    order: group.order,
  })),
];

export const ORISTUDIO_CP_ACTIONS: OristudioCpActionDefinition[] = [
  ...ORISTUDIO_CP_LINE_TYPE_ACTIONS,
  ...ORISTUDIO_CP_COMMANDS.map((command) =>
    command.operationId === 'DrawCreaseFree'
      ? commandAction(command, {
          id: 'cp.action.draw-crease',
          upstreamAction: 'drawCreaseFreeAction',
          upstreamMouseMode: 'DRAW_CREASE_FREE_1',
        })
      : commandAction(command)
  ),
  ...(AUXILIARY_DRAW_COMMAND
    ? [
        {
          ...commandAction(AUXILIARY_DRAW_COMMAND, {
            id: 'cp.action.draw-auxiliary-line',
            label: 'Draw auxiliary line',
            icon: 'scan-line',
            upstreamAction: 'h_senbun_nyuryokuAction',
            upstreamMouseMode: 'DRAW_CREASE_FREE_1',
            tooltip: 'Draw an Oriedita auxiliary line',
            uiStatus: 'not-implemented',
            disabledReason: 'Auxiliary-line insertion mode is not implemented in the CP kernel yet',
            lineInputMode: 'aux-line',
          }),
          placement: 'left-rail-overflow',
        } satisfies OristudioCpCommandActionDefinition,
      ]
    : []),
];

function lineTypeAction(
  id: string,
  label: string,
  railLabel: string,
  lineColor: OristudioCpLineColor,
  upstreamAction: string,
  icon: string
): OristudioCpLineTypeActionDefinition {
  return {
    id: `cp.action.line-type.${id}`,
    kind: 'line-type',
    label,
    railLabel,
    group: 'line-type',
    placement: 'left-rail',
    icon,
    upstreamAction,
    tooltip: `Set active line type to ${label.toLowerCase()}`,
    uiStatus: 'ready',
    disabledReason: 'Ready',
    lineColor,
    lineInputMode: 'fold-line',
  };
}

function commandAction(
  command: OristudioCpCommandDefinition,
  overrides: Partial<OristudioCpCommandActionDefinition> = {}
): OristudioCpCommandActionDefinition {
  return {
    id: actionIdForCommand(command),
    kind: 'command',
    label: command.label,
    group: command.group,
    placement: command.placement,
    icon: command.icon,
    upstreamAction: command.upstream,
    tooltip: command.tooltip,
    uiStatus: command.uiStatus,
    disabledReason: command.disabledReason,
    shortcut: command.shortcut,
    command,
    operationId: command.operationId,
    toolSteps: command.toolSteps,
    inputMode: command.inputMode,
    lineInputMode: 'fold-line',
    repeatable:
      command.operationId === 'DrawCreaseFree' ||
      command.operationId === 'DrawCreaseRestricted',
    ...overrides,
  };
}

function actionIdForCommand(command: OristudioCpCommandDefinition): OristudioCpActionId {
  return `cp.action.${command.id.replace(/^cp\./u, '')}`;
}

export function cpActionsForGroup(group: OristudioCpActionGroupId): OristudioCpActionDefinition[] {
  return ORISTUDIO_CP_ACTIONS.filter((action) => action.group === group);
}

export function cpActionById(
  actionId: OristudioCpActionId
): OristudioCpActionDefinition | undefined {
  return ORISTUDIO_CP_ACTIONS.find((action) => action.id === actionId);
}

export function cpRailActions(): OristudioCpActionDefinition[] {
  return ORISTUDIO_CP_ACTIONS.filter(
    (action) => action.placement === 'left-rail' || action.placement === 'left-rail-overflow'
  );
}
