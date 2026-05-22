import { expose } from 'comlink';
import init, {
  cp_operation_descriptors,
  document_snapshot,
  document_summary,
  execute_cp_command,
  export_cp,
  export_fold,
  free_document,
  load_cp,
  load_document,
  load_fold,
} from '../generated/oristudio-cp-wasm/oristudio_cp_wasm';
import type {
  OristudioCpCommandPayload,
  OristudioCpCommandResult,
  OristudioCpDocumentSnapshot,
  OristudioCpDocumentSummary,
  OristudioCpOperationDescriptor,
} from '../engine/oristudioCpTypes';
import type { OristudioCpOperationId } from '../lib/oristudioCpCommands';
import type { WasmErrorEnvelope } from '../engine/types';

let ready: Promise<void> | null = null;

async function ensureReady() {
  ready ??= init().then(() => undefined);
  await ready;
}

function normalizeError(error: unknown): WasmErrorEnvelope {
  if (
    error &&
    typeof error === 'object' &&
    'code' in error &&
    'message' in error &&
    typeof (error as { code: unknown }).code === 'string'
  ) {
    return error as WasmErrorEnvelope;
  }
  return {
    code: 'oristudio_cp_wasm_error',
    message: error instanceof Error ? error.message : String(error),
  };
}

async function call<T>(fn: () => T): Promise<T> {
  await ensureReady();
  try {
    return fn();
  } catch (error) {
    throw normalizeError(error);
  }
}

const api = {
  async operationDescriptors(): Promise<OristudioCpOperationDescriptor[]> {
    return call(() => cp_operation_descriptors() as OristudioCpOperationDescriptor[]);
  },
  async loadCp(text: string, title = ''): Promise<number> {
    return call(() => load_cp(text, title));
  },
  async loadFold(text: string, title = ''): Promise<number> {
    return call(() => load_fold(text, title));
  },
  async loadDocument(document: OristudioCpDocumentSnapshot): Promise<number> {
    return call(() => load_document(document));
  },
  async snapshot(handle: number): Promise<OristudioCpDocumentSnapshot> {
    return call(() => document_snapshot(handle) as OristudioCpDocumentSnapshot);
  },
  async summary(handle: number): Promise<OristudioCpDocumentSummary> {
    return call(() => document_summary(handle) as OristudioCpDocumentSummary);
  },
  async executeCommand(
    handle: number,
    operationId: OristudioCpOperationId,
    payload: OristudioCpCommandPayload = {}
  ): Promise<OristudioCpCommandResult> {
    return call(() => execute_cp_command(handle, operationId, payload) as OristudioCpCommandResult);
  },
  async exportCp(handle: number): Promise<string> {
    return call(() => export_cp(handle));
  },
  async exportFold(handle: number): Promise<string> {
    return call(() => export_fold(handle));
  },
  async freeDocument(handle: number): Promise<void> {
    return call(() => free_document(handle));
  },
};

export type OristudioCpWorkerApi = typeof api;

expose(api);
