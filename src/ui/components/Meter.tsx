import { Icon } from './Icon';
import type { IconName } from '../icons/paths';

export interface MeterProps {
  label: string;
  /** 0..max; clamped for display. */
  value: number;
  max?: number;
  icon?: IconName;
  /** At or below this value the meter turns danger red. */
  danger?: number;
}

/** A thin horizontal hairline meter with its value in monospace. */
export function Meter({ label, value, max = 100, icon, danger = 20 }: MeterProps) {
  const safeMax = max > 0 ? max : 1;
  const clamped = Math.min(safeMax, Math.max(0, Number.isFinite(value) ? value : 0));
  const shown = Math.round(clamped);
  const fraction = clamped / safeMax;
  const low = clamped <= danger;
  return (
    <div
      class={low ? 'meter meter--low' : 'meter'}
      role="meter"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={safeMax}
      aria-valuenow={shown}
    >
      <div class="meter__head">
        {icon && <Icon name={icon} size={16} />}
        <span class="meter__label">{label}</span>
        <span class="meter__value">{shown}</span>
      </div>
      <svg class="meter__bar" viewBox="0 0 100 4" preserveAspectRatio="none" aria-hidden="true">
        <line class="meter__track" x1="0" y1="2" x2="100" y2="2" vector-effect="non-scaling-stroke" />
        {[25, 50, 75].map((x) => (
          <line
            key={x}
            class="meter__tick"
            x1={x}
            y1="0"
            x2={x}
            y2="4"
            vector-effect="non-scaling-stroke"
          />
        ))}
        {fraction > 0 && (
          <line
            class="meter__fill"
            x1="0"
            y1="2"
            x2={fraction * 100}
            y2="2"
            vector-effect="non-scaling-stroke"
          />
        )}
      </svg>
    </div>
  );
}
