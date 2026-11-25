/**
 * A drag is not a tap. The game frame never scrolls as a whole (and the iOS
 * web view's own scroll view is off), so a finger dragged across a button in
 * the fixed header, the journal strip or the tab bar starts no scroll that
 * would cancel the tap: WebKit would deliver a click when it lifts. This
 * swallows the click after a touch or pen moved further than a tap does.
 * Mouse and keyboard clicks are left alone.
 */
const SLOP = 10;

export function installDragGuard(root: Document = document): () => void {
  let x = 0;
  let y = 0;
  let moved = false;
  /** When a dragged touch lifted; only the click that follows it at once is swallowed. */
  let liftedAt = -Infinity;
  const down = (e: PointerEvent) => {
    x = e.clientX;
    y = e.clientY;
    moved = false;
  };
  const move = (e: PointerEvent) => {
    if (e.pointerType === 'mouse') return;
    if (Math.abs(e.clientX - x) > SLOP || Math.abs(e.clientY - y) > SLOP) moved = true;
  };
  const up = () => {
    if (moved) liftedAt = performance.now();
    moved = false;
  };
  const cancel = () => {
    moved = false;
  };
  const click = (e: MouseEvent) => {
    if (performance.now() - liftedAt > 600) return;
    liftedAt = -Infinity;
    e.preventDefault();
    e.stopPropagation();
  };
  root.addEventListener('pointerdown', down, true);
  root.addEventListener('pointermove', move, true);
  root.addEventListener('pointerup', up, true);
  root.addEventListener('pointercancel', cancel, true);
  root.addEventListener('click', click, true);
  return () => {
    root.removeEventListener('pointerup', up, true);
    root.removeEventListener('pointercancel', cancel, true);
    root.removeEventListener('pointerdown', down, true);
    root.removeEventListener('pointermove', move, true);
    root.removeEventListener('click', click, true);
  };
}
