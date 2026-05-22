import type {
  OristudioCpCommandDefinition,
  OristudioCpCommandUiStatus,
  OristudioCpOperationId,
} from './oristudioCpCommands';

export type OristudioCpToolPhase = 'idle' | 'active' | 'blocked' | 'error';

export interface OristudioCpToolState {
  activeOperationId: OristudioCpOperationId | null;
  phase: OristudioCpToolPhase;
  prompt: string;
  status: OristudioCpCommandUiStatus | 'idle' | 'error';
  stepIndex: number;
  steps: readonly string[];
}

export type OristudioCpToolEvent =
  | { type: 'selectCommand'; command: OristudioCpCommandDefinition; editable: boolean }
  | { type: 'advanceStep' }
  | { type: 'commit' }
  | { type: 'cancel' }
  | { type: 'reset' }
  | { type: 'commandError'; message: string };

export const IDLE_ORISTUDIO_CP_TOOL_STATE: OristudioCpToolState = {
  activeOperationId: null,
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
    case 'selectCommand':
      return stateForCommand(event.command, event.editable);
    case 'advanceStep':
      return advanceToolStep(state);
    case 'commit':
    case 'cancel':
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
    activeOperationId: command.operationId,
    phase: 'active',
    prompt: steps.length > 0 ? `${command.label}: ${steps[0]}` : `Tool ${command.label}`,
    status: command.uiStatus,
    stepIndex: 0,
    steps,
  };
}

function blockedCommandState(
  command: OristudioCpCommandDefinition,
  reason: string
): OristudioCpToolState {
  return {
    activeOperationId: command.operationId,
    phase: 'blocked',
    prompt: `${command.label}: ${reason}`,
    status: command.uiStatus,
    stepIndex: 0,
    steps: command.toolSteps ?? [],
  };
}

function advanceToolStep(state: OristudioCpToolState): OristudioCpToolState {
  if (state.phase !== 'active' || state.steps.length === 0) return state;
  const nextStepIndex = Math.min(state.stepIndex + 1, state.steps.length - 1);
  if (nextStepIndex === state.stepIndex) return state;
  return {
    ...state,
    stepIndex: nextStepIndex,
    prompt: `${activeToolLabel(state)}: ${state.steps[nextStepIndex]}`,
  };
}

function activeToolLabel(state: OristudioCpToolState): string {
  if (!state.activeOperationId) return 'Tool';
  return state.prompt.split(':', 1)[0] || 'Tool';
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
