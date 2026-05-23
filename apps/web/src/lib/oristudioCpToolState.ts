import type {
  OristudioCpActionDefinition,
  OristudioCpActionId,
} from './oristudioCpActions';
import type {
  OristudioCpCommandDefinition,
  OristudioCpCommandUiStatus,
  OristudioCpOperationId,
} from './oristudioCpCommands';

export type OristudioCpToolPhase = 'idle' | 'active' | 'blocked' | 'error';

export interface OristudioCpToolState {
  activeActionId: OristudioCpActionId | null;
  activeOperationId: OristudioCpOperationId | null;
  activeLabel: string | null;
  phase: OristudioCpToolPhase;
  prompt: string;
  status: OristudioCpCommandUiStatus | 'idle' | 'error';
  stepIndex: number;
  steps: readonly string[];
}

export type OristudioCpToolEvent =
  | { type: 'selectAction'; action: OristudioCpActionDefinition; editable: boolean }
  | { type: 'selectCommand'; command: OristudioCpCommandDefinition; editable: boolean }
  | { type: 'advanceStep' }
  | { type: 'commit'; keepActive?: boolean }
  | { type: 'cancel'; keepActive?: boolean }
  | { type: 'reset' }
  | { type: 'commandError'; message: string };

export const IDLE_ORISTUDIO_CP_TOOL_STATE: OristudioCpToolState = {
  activeActionId: null,
  activeOperationId: null,
  activeLabel: null,
  phase: 'idle',
  prompt: 'Tool Select',
  status: 'idle',
  stepIndex: 0,
  steps: [],
};

export function transitionOristudioCpToolState(
  state: OristudioCpToolState,
  event: OristudioCpToolEvent
): OristudioCpToolState {
  switch (event.type) {
    case 'selectAction':
      return stateForAction(event.action, event.editable);
    case 'selectCommand':
      return stateForCommand(event.command, event.editable);
    case 'advanceStep':
      return advanceToolStep(state);
    case 'commit':
    case 'cancel':
      if (event.keepActive) return resetActiveToolInput(state);
      return IDLE_ORISTUDIO_CP_TOOL_STATE;
    case 'reset':
      return IDLE_ORISTUDIO_CP_TOOL_STATE;
    case 'commandError':
      if (!state.activeOperationId) return IDLE_ORISTUDIO_CP_TOOL_STATE;
      return {
        ...state,
        phase: 'error',
        prompt: `${activeToolLabel(state)}: ${event.message}`,
        status: 'error',
      };
  }
}

export function cancelOristudioCpToolState(state: OristudioCpToolState): {
  handled: boolean;
  state: OristudioCpToolState;
} {
  if (state.phase === 'idle') return { handled: false, state };
  return { handled: true, state: IDLE_ORISTUDIO_CP_TOOL_STATE };
}

function stateForCommand(
  command: OristudioCpCommandDefinition,
  editable: boolean
): OristudioCpToolState {
  if (!editable) {
    return blockedCommandState(command, 'Open an editable crease pattern first');
  }
  if (command.uiStatus !== 'ready') {
    return blockedCommandState(command, commandUiStatusLabel(command.uiStatus));
  }

  const steps = command.toolSteps ?? [];
  return {
    activeActionId: null,
    activeOperationId: command.operationId,
    activeLabel: command.label,
    phase: 'active',
    prompt: promptForStep(command.label, steps, 0),
    status: command.uiStatus,
    stepIndex: 0,
    steps,
  };
}

function stateForAction(
  action: OristudioCpActionDefinition,
  editable: boolean
): OristudioCpToolState {
  if (action.kind === 'line-type') return IDLE_ORISTUDIO_CP_TOOL_STATE;
  if (!editable) {
    return blockedActionState(action, 'Open an editable crease pattern first');
  }
  if (action.uiStatus !== 'ready') {
    return blockedActionState(action, commandUiStatusLabel(action.uiStatus));
  }

  const steps = action.toolSteps ?? action.command.toolSteps ?? [];
  return {
    activeActionId: action.id,
    activeOperationId: action.operationId,
    activeLabel: action.label,
    phase: 'active',
    prompt: promptForStep(action.label, steps, 0),
    status: action.uiStatus,
    stepIndex: 0,
    steps,
  };
}

function blockedCommandState(
  command: OristudioCpCommandDefinition,
  reason: string
): OristudioCpToolState {
  return {
    activeActionId: null,
    activeOperationId: command.operationId,
    activeLabel: command.label,
    phase: 'blocked',
    prompt: `${command.label}: ${reason}`,
    status: command.uiStatus,
    stepIndex: 0,
    steps: command.toolSteps ?? [],
  };
}

function blockedActionState(
  action: OristudioCpActionDefinition & { kind: 'command' },
  reason: string
): OristudioCpToolState {
  const steps = action.toolSteps ?? action.command.toolSteps ?? [];
  return {
    activeActionId: action.id,
    activeOperationId: action.operationId,
    activeLabel: action.label,
    phase: 'blocked',
    prompt: `${action.label}: ${reason}`,
    status: action.uiStatus,
    stepIndex: 0,
    steps,
  };
}

function advanceToolStep(state: OristudioCpToolState): OristudioCpToolState {
  if (state.phase !== 'active' || state.steps.length === 0) return state;
  const nextStepIndex = Math.min(state.stepIndex + 1, state.steps.length - 1);
  if (nextStepIndex === state.stepIndex) return state;
  return {
    ...state,
    stepIndex: nextStepIndex,
    prompt: promptForStep(activeToolLabel(state), state.steps, nextStepIndex),
  };
}

function resetActiveToolInput(state: OristudioCpToolState): OristudioCpToolState {
  if (state.phase === 'idle' || !state.activeOperationId) return IDLE_ORISTUDIO_CP_TOOL_STATE;
  return {
    ...state,
    phase: state.status === 'ready' ? 'active' : state.phase,
    stepIndex: 0,
    prompt: promptForStep(activeToolLabel(state), state.steps, 0),
  };
}

function activeToolLabel(state: OristudioCpToolState): string {
  if (state.activeLabel) return state.activeLabel;
  if (!state.activeOperationId) return 'Tool';
  return state.prompt.split(':', 1)[0] || 'Tool';
}

function promptForStep(label: string, steps: readonly string[], stepIndex: number): string {
  return steps.length > 0 ? `${label}: ${steps[stepIndex]}` : `Tool ${label}`;
}

function commandUiStatusLabel(status: OristudioCpCommandUiStatus): string {
  switch (status) {
    case 'porting':
      return 'Porting';
    case 'out-of-scope-ui':
      return 'Out of scope';
    case 'ready':
      return 'Ready';
    case 'not-implemented':
      return 'Not implemented';
  }
}
