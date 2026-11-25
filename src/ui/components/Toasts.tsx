import type { Toast } from '../state/store';
import { Icon } from './Icon';

/** Quiet notices at the top of the column: marks, skill gains, refusals. */
export function Toasts({ toasts, onDismiss }: { toasts: Toast[]; onDismiss: (id: number) => void }) {
  return (
    <div class="toasts" role="status" aria-live="polite" aria-atomic="false">
      {toasts.map((t) => (
        <div key={t.id} class={`toast toast--${t.tone}`}>
          <Icon name={t.icon} size={24} class="toast__icon" />
          <div class="toast__body">
            <p class="toast__title">{t.title}</p>
            {t.text && <p class="toast__text">{t.text}</p>}
          </div>
          <button type="button" class="toast__close" aria-label="Dismiss" onClick={() => onDismiss(t.id)}>
            <Icon name="close" size={16} />
          </button>
        </div>
      ))}
    </div>
  );
}
