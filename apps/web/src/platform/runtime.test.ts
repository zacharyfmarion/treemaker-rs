import { describe, expect, it } from 'vitest';
import { getRuntimeSurface, isDesktopRuntime, isWebRuntime } from './runtime';

describe('runtime detection', () => {
  it('defaults to web without Tauri globals', () => {
    expect(getRuntimeSurface({})).toBe('web');
    expect(isWebRuntime({})).toBe(true);
  });

  it('detects Tauri internals as desktop', () => {
    const host = { __TAURI_INTERNALS__: {} };
    expect(getRuntimeSurface(host)).toBe('desktop');
    expect(isDesktopRuntime(host)).toBe(true);
  });

  it('detects explicit Tauri flags as desktop', () => {
    expect(getRuntimeSurface({ isTauri: true })).toBe('desktop');
    expect(getRuntimeSurface({ __TAURI__: {} })).toBe('desktop');
  });
});
