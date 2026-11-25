import { Icon } from './Icon';

export interface Tab<T extends string> {
  id: T;
  label: string;
  icon: string;
  /** A small marker: something here wants attention. */
  flag?: boolean;
  /** The marker is news (bone), not danger (red). */
  quiet?: boolean;
}

export interface TabBarProps<T extends string> {
  tabs: Tab<T>[];
  current: T;
  onChange: (id: T) => void;
  label: string;
  /** Prefix for tab / panel ids. */
  idBase: string;
}

/** Bottom tab bar (ARIA tabs). Arrow keys move between tabs. */
export function TabBar<T extends string>({ tabs, current, onChange, label, idBase }: TabBarProps<T>) {
  const move = (dir: number) => {
    const i = tabs.findIndex((t) => t.id === current);
    const next = tabs[(i + dir + tabs.length) % tabs.length];
    if (!next) return;
    onChange(next.id);
    document.getElementById(`${idBase}-tab-${next.id}`)?.focus();
  };
  return (
    <nav class="tabbar">
      <div
        class="tabbar__list"
        role="tablist"
        aria-label={label}
        onKeyDown={(e) => {
          if (e.key === 'ArrowRight') {
            e.preventDefault();
            move(1);
          } else if (e.key === 'ArrowLeft') {
            e.preventDefault();
            move(-1);
          }
        }}
      >
        {tabs.map((t) => {
          const on = t.id === current;
          return (
            <button
              key={t.id}
              type="button"
              role="tab"
              id={`${idBase}-tab-${t.id}`}
              aria-selected={on}
              aria-controls={`${idBase}-panel`}
              tabIndex={on ? 0 : -1}
              class={on ? 'tab tab--on' : 'tab'}
              onClick={() => onChange(t.id)}
            >
              <Icon name={t.icon} size={20} />
              <span class="tab__label">{t.label}</span>
              {t.flag && (
                <span
                  class={t.quiet ? 'tab__flag tab__flag--quiet' : 'tab__flag'}
                  aria-label={t.quiet ? 'something new' : 'needs attention'}
                />
              )}
            </button>
          );
        })}
      </div>
    </nav>
  );
}
