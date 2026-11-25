export interface HairlineProps {
  value: number;
  max: number;
  /** Accessible name. */
  label: string;
  /** Draw in the danger accent. */
  danger?: boolean;
  /** Tick marks at each whole step (for small maxima such as levels). */
  steps?: boolean;
}

/** A bare progress hairline: skills, stations, codex, ferry. */
export function Hairline({ value, max, label, danger = false, steps = false }: HairlineProps) {
  const safeMax = max > 0 ? max : 1;
  const v = Math.min(safeMax, Math.max(0, Number.isFinite(value) ? value : 0));
  const f = (v / safeMax) * 100;
  const ticks = steps && safeMax <= 12 ? Array.from({ length: safeMax - 1 }, (_, i) => ((i + 1) / safeMax) * 100) : [];
  return (
    <svg
      class={danger ? 'hairline hairline--danger' : 'hairline'}
      viewBox="0 0 100 4"
      preserveAspectRatio="none"
      role="progressbar"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={safeMax}
      aria-valuenow={Math.round(v)}
    >
      <line class="hairline__track" x1="0" y1="2" x2="100" y2="2" vector-effect="non-scaling-stroke" />
      {ticks.map((x) => (
        <line key={x} class="hairline__tick" x1={x} y1="0" x2={x} y2="4" vector-effect="non-scaling-stroke" />
      ))}
      {f > 0 && <line class="hairline__fill" x1="0" y1="2" x2={f} y2="2" vector-effect="non-scaling-stroke" />}
    </svg>
  );
}
