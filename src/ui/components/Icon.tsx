import { iconPath, type IconName } from '../icons/paths';

export interface IconProps {
  /** An icon key; keys from the core are plain strings and fall back to `item`. */
  name: IconName | (string & {});
  /** Accessible name. Omit for decorative icons next to visible text. */
  label?: string;
  /** Rendered size in CSS pixels (the grid is always 24). */
  size?: 16 | 20 | 24 | 32 | 48 | 64;
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
      <path d={iconPath(name)} />
    </svg>
  );
}
