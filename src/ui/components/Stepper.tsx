import { Icon } from './Icon';

export interface StepperProps {
  value: number;
  min: number;
  max: number;
  onChange: (n: number) => void;
  label: string;
}

/** A quantity stepper: minus, the number, plus. Arrow keys work on the group. */
export function Stepper({ value, min, max, onChange, label }: StepperProps) {
  const set = (n: number) => onChange(Math.min(max, Math.max(min, n)));
  return (
    <div
      class="stepper"
      role="group"
      aria-label={label}
      onKeyDown={(e) => {
        if (e.key === 'ArrowUp' || e.key === 'ArrowRight') {
          e.preventDefault();
          set(value + 1);
        } else if (e.key === 'ArrowDown' || e.key === 'ArrowLeft') {
          e.preventDefault();
          set(value - 1);
        } else if (e.key === 'End') {
          e.preventDefault();
          set(max);
        } else if (e.key === 'Home') {
          e.preventDefault();
          set(min);
        }
      }}
    >
      <button type="button" class="stepper__btn" aria-label="One fewer" disabled={value <= min} onClick={() => set(value - 1)}>
        <Icon name="minus" size={16} />
      </button>
      <output class="stepper__value num" aria-live="polite">
        {value}
        <span class="stepper__max">/{max}</span>
      </output>
      <button type="button" class="stepper__btn" aria-label="One more" disabled={value >= max} onClick={() => set(value + 1)}>
        <Icon name="plus" size={16} />
      </button>
    </div>
  );
}
