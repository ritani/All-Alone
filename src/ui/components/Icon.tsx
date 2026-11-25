import { ICON_PATHS, type IconName } from '../icons/paths';

export interface IconProps {
  name: IconName;
  /** Accessible name. Omit for decorative icons next to visible text. */
  label?: string;
  /** Rendered size in CSS pixels (the grid is always 24). */
  size?: 16 | 20 | 24 | 32 | 48;
  class?: string;
}

/** Stroke-only line icon, drawn in currentColor. */
export function Icon({ name, label, size = 24, class: extra }: IconProps) {
  return (
    <svg
      class={extra ? `icon ${extra}` : 'icon'}
      viewBox="0 0 24 24"
      width={size}
      height={size}
      fill="none"
      stroke="currentColor"
      stroke-width="1.25"
      stroke-linecap="square"
      stroke-linejoin="miter"
      stroke-miterlimit="10"
      role={label ? 'img' : undefined}
      aria-label={label}
      aria-hidden={label ? undefined : 'true'}
      focusable="false"
    >
      <path d={ICON_PATHS[name]} />
    </svg>
  );
}
