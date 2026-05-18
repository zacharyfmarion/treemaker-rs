export type RuntimeSurface = 'web' | 'desktop';

type RuntimeHost = Record<string, unknown>;

const TAURI_INTERNALS_KEY = '__TAURI_INTERNALS__';
const TAURI_V1_KEY = '__TAURI__';
const TAURI_FLAG_KEY = 'isTauri';

function defaultHost(): RuntimeHost | undefined {
  if (typeof window !== 'undefined') return window as unknown as RuntimeHost;
  if (typeof globalThis !== 'undefined') return globalThis as RuntimeHost;
  return undefined;
}

export function getRuntimeSurface(host: RuntimeHost | undefined = defaultHost()): RuntimeSurface {
  if (!host) return 'web';
  return TAURI_INTERNALS_KEY in host || TAURI_V1_KEY in host || host[TAURI_FLAG_KEY] === true
    ? 'desktop'
    : 'web';
}

export function isDesktopRuntime(host?: RuntimeHost): boolean {
  return getRuntimeSurface(host) === 'desktop';
}

export function isWebRuntime(host?: RuntimeHost): boolean {
  return getRuntimeSurface(host) === 'web';
}
