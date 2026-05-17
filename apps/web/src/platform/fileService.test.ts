import { describe, expect, it } from 'vitest';
import { createFileService } from './fileService';

describe('file service selection', () => {
  it('creates a browser service for web runtime', async () => {
    const service = createFileService('web');
    expect(service.surface).toBe('web');
    expect(service.supportsNativeDialogs).toBe(false);
    await expect(service.run('saveProject')).resolves.toMatchObject({
      status: 'not_implemented',
    });
  });

  it('creates a Tauri service for desktop runtime', async () => {
    const service = createFileService('desktop');
    expect(service.surface).toBe('desktop');
    expect(service.supportsNativeDialogs).toBe(true);
    await expect(service.run('openProject')).resolves.toMatchObject({
      status: 'not_implemented',
    });
  });
});
