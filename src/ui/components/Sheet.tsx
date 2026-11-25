import type { ComponentChildren } from 'preact';
import { useEffect, useRef } from 'preact/hooks';
import { useBack } from '../back';
import { Icon } from './Icon';

export interface SheetProps {
  title: string;
  onClose: () => void;
  children: ComponentChildren;
  /** Nearly the full height of the screen (the journal, long lists). */
  tall?: boolean;
  /** A control drawn before the title (a back arrow inside the sheet). */
  lead?: ComponentChildren;
  /** Called with the body element once it is drawn (the journal scrolls to its end). */
  bodyRef?: (el: HTMLDivElement | null) => void;
  /** Back (Escape, the Android back button) steps back inside the sheet instead of closing it. */
  onBack?: (() => void) | undefined;
}

const FOCUSABLE = 'button:not([disabled]), [href], input, select, textarea, [tabindex]:not([tabindex="-1"])';

/** Open sheets. While any is open, nothing behind it scrolls. */
let open = 0;
function lockBackground(on: boolean): void {
  open = Math.max(0, open + (on ? 1 : -1));
  document.documentElement.classList.toggle('has-sheet', open > 0);
}

/**
 * A modal sheet rising from the bottom of the column. Focus moves into it,
 * stays inside while open, and returns to where it was on close. Escape (and
 * the Android back button) closes. Its body is the only thing that scrolls
 * while it is open.
 */
export function Sheet({ title, onClose, children, tall = false, lead, bodyRef, onBack }: SheetProps) {
  const ref = useRef<HTMLDivElement>(null);
  const back = onBack ?? onClose;
  useBack(true, back);
  useEffect(() => {
    const before = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    ref.current?.querySelector<HTMLElement>('.sheet__title')?.focus({ preventScroll: true });
    lockBackground(true);
    return () => {
      lockBackground(false);
      before?.focus({ preventScroll: true });
    };
  }, []);
  // A new title (a page inside the sheet) takes the focus again.
  const firstTitle = useRef(title);
  useEffect(() => {
    if (firstTitle.current === title) return;
    firstTitle.current = title;
    ref.current?.querySelector<HTMLElement>('.sheet__title')?.focus({ preventScroll: true });
    ref.current?.querySelector('.sheet__body')?.scrollTo(0, 0);
  }, [title]);

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      e.stopPropagation();
      back();
      return;
    }
    if (e.key !== 'Tab' || !ref.current) return;
    e.stopPropagation();
    const items = [...ref.current.querySelectorAll<HTMLElement>(FOCUSABLE)];
    const first = items[0];
    const last = items[items.length - 1];
    if (!first || !last) {
      e.preventDefault();
      return;
    }
    const at = document.activeElement;
    if (e.shiftKey && (at === first || !ref.current.contains(at))) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && (at === last || !ref.current.contains(at))) {
      e.preventDefault();
      first.focus();
    }
  };

  return (
    <div class="sheet-scrim" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div
        class={tall ? 'sheet sheet--tall' : 'sheet'}
        role="dialog"
        aria-modal="true"
        aria-label={title}
        ref={ref}
        onKeyDown={onKeyDown}
      >
        <header class="sheet__head">
          {lead}
          <h2 class="sheet__title" tabIndex={-1}>
            {title}
          </h2>
          <button type="button" class="sheet__close" aria-label="Close" onClick={onClose}>
            <Icon name="close" size={20} />
          </button>
        </header>
        <div class="sheet__body" ref={(el) => bodyRef?.(el)}>
          {children}
        </div>
      </div>
    </div>
  );
}
