import { describe, expect, it } from 'vitest';
import { popBack, pushBack } from './back';

describe('back stack', () => {
  it('closes the newest layer first, then the one under it, then nothing', () => {
    const closed: string[] = [];
    // A sheet (the journal), then a page inside a sheet (a codex entry) above it.
    const removeSheet = pushBack(() => closed.push('sheet'));
    const removeEntry = pushBack(() => closed.push('entry'));
    expect(popBack()).toBe(true);
    expect(closed).toEqual(['entry']);
    removeEntry();
    expect(popBack()).toBe(true);
    expect(closed).toEqual(['entry', 'sheet']);
    removeSheet();
    // Nothing open: the store goes back a screen instead.
    expect(popBack()).toBe(false);
  });

  it('forgets a layer that closed by itself', () => {
    const closed: string[] = [];
    const under = pushBack(() => closed.push('under'));
    const over = pushBack(() => closed.push('over'));
    over();
    expect(popBack()).toBe(true);
    expect(closed).toEqual(['under']);
    under();
    expect(popBack()).toBe(false);
  });
});
