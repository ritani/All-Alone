// @vitest-environment node
import { describe, expect, it } from 'vitest';
import { installDragGuard } from './drag';

type Handler = (e: never) => void;

/** A stand-in for `document` that records listeners and dispatches by hand. */
function fakeRoot() {
  const on = new Map<string, Handler>();
  const root = {
    addEventListener: (type: string, fn: Handler) => on.set(type, fn),
    removeEventListener: (type: string) => on.delete(type),
  };
  const fire = (type: string, init: Record<string, unknown> = {}) => {
    let stopped = false;
    const e = { pointerType: 'touch', clientX: 0, clientY: 0, preventDefault: () => undefined, stopPropagation: () => (stopped = true), ...init };
    on.get(type)?.(e as never);
    return stopped;
  };
  return { root: root as unknown as Document, fire, on };
}

describe('drag guard', () => {
  it('lets a tap through and swallows the click after a drag', () => {
    const { root, fire } = fakeRoot();
    installDragGuard(root);
    fire('pointerdown', { clientX: 10, clientY: 10 });
    fire('pointermove', { clientX: 12, clientY: 13 });
    fire('pointerup');
    expect(fire('click')).toBe(false);

    fire('pointerdown', { clientX: 10, clientY: 10 });
    fire('pointermove', { clientX: 10, clientY: 80 });
    fire('pointerup');
    expect(fire('click')).toBe(true);
    // The next click (a key press) is not touched.
    expect(fire('click')).toBe(false);
  });

  it('ignores mouse movement and removes its listeners', () => {
    const { root, fire, on } = fakeRoot();
    const undo = installDragGuard(root);
    fire('pointerdown', { pointerType: 'mouse', clientX: 0, clientY: 0 });
    fire('pointermove', { pointerType: 'mouse', clientX: 50, clientY: 50 });
    fire('pointerup', { pointerType: 'mouse' });
    expect(fire('click')).toBe(false);
    undo();
    expect(on.size).toBe(0);
  });
});
