/**
 * What "back" (the Android back button) closes first. Open sheets and other
 * in-screen layers register a closer while they are shown; the newest one
 * wins. When nothing is registered, the store goes back a screen.
 */
import { useEffect, useRef } from 'preact/hooks';

const stack: { close: () => void }[] = [];

/** Register a closer; returns its removal. */
export function pushBack(close: () => void): () => void {
  const entry = { close };
  stack.push(entry);
  return () => {
    const i = stack.lastIndexOf(entry);
    if (i >= 0) stack.splice(i, 1);
  };
}

/** Close the newest layer. False when there was none. */
export function popBack(): boolean {
  const top = stack[stack.length - 1];
  if (!top) return false;
  top.close();
  return true;
}

/** While `active`, back calls `close` (always the latest one passed). */
export function useBack(active: boolean, close: () => void): void {
  const ref = useRef(close);
  ref.current = close;
  useEffect(() => (active ? pushBack(() => ref.current()) : undefined), [active]);
}
