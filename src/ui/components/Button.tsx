import type { ComponentChildren } from 'preact';
import { Icon } from './Icon';
import type { IconName } from '../icons/paths';

export interface ButtonProps {
  children: ComponentChildren;
  onClick?: (() => void) | undefined;
  /** `primary` is outlined in bone, `quiet` is text-only, `danger` is for real danger. */
  variant?: 'primary' | 'quiet' | 'danger';
  icon?: IconName;
  disabled?: boolean;
  block?: boolean;
  type?: 'button' | 'submit';
}

export function Button({
  children,
  onClick,
  variant = 'primary',
  icon,
  disabled = false,
  block = false,
  type = 'button',
}: ButtonProps) {
  const cls = ['btn', `btn--${variant}`, block ? 'btn--block' : ''].filter(Boolean).join(' ');
  return (
    <button type={type} class={cls} disabled={disabled} onClick={onClick}>
      {icon && <Icon name={icon} size={20} />}
      <span class="btn__label">{children}</span>
    </button>
  );
}
