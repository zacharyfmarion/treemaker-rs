import type { MenuActionId } from '../commands/menuActions';
import type { DocumentMode } from '../lib/sampleProject';
import type { OristudioCpActionId } from '../lib/oristudioCpActions';
import {
  handleShortcutKeyDown,
  type ShortcutExecutors,
} from './shortcutDispatcher';
import type {
  ShortcutOverrides,
  ShortcutScope,
  ViewportShortcutId,
} from './shortcuts';

type CpActionExecutor = (id: OristudioCpActionId) => unknown;
type ViewportExecutor = (id: ViewportShortcutId) => unknown;

const viewportExecutors: Partial<Record<DocumentMode, ViewportExecutor>> = {};
let cpActionExecutor: CpActionExecutor | null = null;
let activeViewportSurface: DocumentMode | null = null;

export interface ShortcutRuntimeContext {
  documentMode: DocumentMode;
  activeEditingSurface: DocumentMode;
  activeViewportSurface?: DocumentMode | null;
}

export interface ShortcutRuntimeOptions {
  context: ShortcutRuntimeContext;
  overrides?: ShortcutOverrides;
  menu: (id: MenuActionId) => unknown;
}

export function registerViewportShortcutExecutor(
  surface: DocumentMode,
  executor: ViewportExecutor
): () => void {
  viewportExecutors[surface] = executor;
  return () => {
    if (viewportExecutors[surface] === executor) {
      delete viewportExecutors[surface];
    }
  };
}

export function registerCpActionShortcutExecutor(executor: CpActionExecutor): () => void {
  cpActionExecutor = executor;
  return () => {
    if (cpActionExecutor === executor) {
      cpActionExecutor = null;
    }
  };
}

export function setActiveShortcutViewportSurface(surface: DocumentMode): void {
  activeViewportSurface = surface;
}

function resolvedViewportSurface(context: ShortcutRuntimeContext): DocumentMode {
  return context.activeViewportSurface ?? activeViewportSurface ?? context.activeEditingSurface;
}

export function shortcutScopeStackForContext(
  context: ShortcutRuntimeContext
): ShortcutScope[] {
  const scopes: ShortcutScope[] = ['viewport'];
  if (
    context.documentMode === 'crease-pattern' &&
    context.activeEditingSurface === 'crease-pattern'
  ) {
    scopes.push('crease-pattern');
  }

  scopes.push('global');
  return scopes;
}

export function handleShortcutRuntimeKeyDown(
  event: KeyboardEvent,
  options: ShortcutRuntimeOptions
): boolean {
  const executors: ShortcutExecutors = {
    menu: options.menu,
    viewport: viewportExecutors[resolvedViewportSurface(options.context)],
  };

  if (
    options.context.documentMode === 'crease-pattern' &&
    options.context.activeEditingSurface === 'crease-pattern' &&
    cpActionExecutor
  ) {
    executors.cpAction = cpActionExecutor;
  }

  return handleShortcutKeyDown(event, {
    scopeStack: shortcutScopeStackForContext(options.context),
    overrides: options.overrides,
    executors,
  });
}
