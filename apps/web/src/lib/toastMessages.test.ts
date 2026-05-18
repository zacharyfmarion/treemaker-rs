import { describe, expect, it } from 'vitest';
import { formatUnknownError, toastFromProjectMessage } from './toastMessages';

describe('toast message helpers', () => {
  it('maps project lifecycle messages to sonner toast content', () => {
    expect(toastFromProjectMessage('Saved crane.tmd5')).toMatchObject({
      kind: 'success',
      title: 'Project saved',
      message: 'Saved crane.tmd5',
    });
    expect(toastFromProjectMessage('Optimize scale')).toMatchObject({
      kind: 'success',
      title: 'Optimization complete',
    });
    expect(toastFromProjectMessage('Copied 2 nodes and 1 edges')).toMatchObject({
      kind: 'info',
      title: 'Selection copied',
    });
  });

  it('formats error-like values for global error toasts', () => {
    expect(formatUnknownError(new Error('boom'))).toBe('boom');
    expect(formatUnknownError({ message: 'bad fold' })).toBe('bad fold');
    expect(formatUnknownError('plain failure')).toBe('plain failure');
  });
});
