import type { ComponentChildren } from 'preact';
import { useEffect, useRef } from 'preact/hooks';
import { useBack } from '../back';
import { Icon } from './Icon';

export interface SheetProps {
  title: string;
  onClose: () => void;
  children: ComponentChildren;
}

const FOCUSABLE = 'button:not([disabled]), [href], input, select, textarea, [tabindex]:not([tabindex="-1"])';

/**
 * A modal sheet rising from the bottom of the column. Focus moves into it,
 * stays inside while open, and returns to where it was on close. Escape (and
 * the Android back button) closes.
 */
export function Sheet({ title, onClose, children }: SheetProps) {
  const ref = useRef<HTMLDivElement>(null);
  useBack(true, onClose);
  useEffect(() => {
    const before = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    ref.current?.querySelector<HTMLElement>('.sheet__title')?.focus();
    return () => before?.focus();
  }, []);

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      e.stopPropagation();
      onClose();
      return;
    }
    if (e.key !== 'Tab' || !ref.current) return;
    const items = [...ref.current.querySelectorAll<HTMLElement>(FOCUSABLE)];
    const first = items[0];
    const last = items[items.length - 1];
    if (!first || !last) return;
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  };

  return (
    <div class="sheet-scrim" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div class="sheet" role="dialog" aria-modal="true" aria-label={title} ref={ref} onKeyDown={onKeyDown}>
        <header class="sheet__head">
          <h2 class="sheet__title label" tabIndex={-1}>
            {title}
          </h2>
          <button type="button" class="sheet__close" aria-label="Close" onClick={onClose}>
            <Icon name="close" size={20} />
          </button>
        </header>
        <div class="sheet__body">{children}</div>
      </div>
    </div>
  );
}
