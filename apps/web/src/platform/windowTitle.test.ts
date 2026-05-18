import { describe, expect, it } from 'vitest';
import { formatWindowTitle } from './windowTitle';

describe('window title formatting', () => {
  it('formats clean web titles', () => {
    expect(
      formatWindowTitle({ projectTitle: 'Crane base', dirty: false, surface: 'web' })
    ).toBe('Crane base - TreeMaker Web');
  });

  it('marks dirty desktop titles', () => {
    expect(
      formatWindowTitle({ projectTitle: 'Crane base', dirty: true, surface: 'desktop' })
    ).toBe('*Crane base - TreeMaker');
  });
});
