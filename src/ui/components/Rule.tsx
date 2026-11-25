import type { ComponentChildren } from 'preact';

export interface RuleProps {
  /** Section label, set in small tracked capitals. Omit for a plain hairline. */
  children?: ComponentChildren;
  level?: 2 | 3;
}

/** A hairline rule, or a section header that runs into a hairline. */
export function Rule({ children, level = 2 }: RuleProps) {
  if (children === undefined) return <hr class="rule" />;
  const Tag = level === 2 ? 'h2' : 'h3';
  return (
    <Tag class="rule rule--label">
      <span class="rule__text">{children}</span>
    </Tag>
  );
}
