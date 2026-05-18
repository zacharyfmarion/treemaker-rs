import { describe, expect, it } from 'vitest';
import { formatUnknownError } from './toastMessages';

describe('toast message helpers', () => {
  it('formats error-like values for global error toasts', () => {
    expect(formatUnknownError(new Error('boom'))).toBe('boom');
    expect(formatUnknownError({ message: 'bad fold' })).toBe('bad fold');
    expect(formatUnknownError('plain failure')).toBe('plain failure');
  });
});
