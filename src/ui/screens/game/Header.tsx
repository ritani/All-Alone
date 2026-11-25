import { useState } from 'preact/hooks';
import type { VitalView, View } from '../../../core';
import { Hairline } from '../../components/Hairline';
import { Icon } from '../../components/Icon';
import { Sheet } from '../../components/Sheet';
import { duration } from '../../format';

function Vital({ v }: { v: VitalView }) {
  const critical = v.level === 'critical';
  const low = v.level === 'low';
  const cls = ['vital', critical ? 'vital--critical' : '', low ? 'vital--low' : ''].filter(Boolean).join(' ');
  return (
    <div class={cls}>
      <Icon name={v.icon} size={16} class="vital__icon" />
      <span class="visually-hidden">{v.name}</span>
      <Hairline value={v.value} max={100} label={`${v.name} ${v.value}`} danger={critical} />
      <span class="vital__value num" aria-hidden="true">
        {v.value}
      </span>
    </div>
  );
}

function daylight(view: View): string {
  if (view.dark || view.daylight_left === null) return 'Dark';
  return `${duration(view.daylight_left)} light`;
}

/** The compact always-visible status: time, weather, noise, vitals, conditions. */
export function Header({ view }: { view: View }) {
  const [open, setOpen] = useState(false);
  const loud = view.noise.visit_chance >= 35;
  return (
    <header class="hud">
      <button type="button" class="hud__top" aria-label="Status in full" onClick={() => setOpen(true)}>
        <span class="hud__day num">Day {view.day}</span>
        <span class="hud__clock num">{view.clock}</span>
        <span class={view.dark ? 'hud__light hud__light--dark label' : 'hud__light label'}>{daylight(view)}</span>
        <span class="hud__weather">
          <Icon name={view.weather.today.icon} size={16} />
          <span class="num">{view.season.temperature}&#176;</span>
        </span>
      </button>

      <div class={loud ? 'hud__noise hud__noise--loud' : 'hud__noise'}>
        <Icon name="noise" size={16} class="hud__noise-icon" />
        <span class="label hud__noise-label">{view.noise.label}</span>
        <Hairline value={view.noise.value} max={100} label={`Noise ${view.noise.value}`} danger={loud} />
        <span class="num hud__noise-figure" title="Chance something comes tonight">
          {view.noise.value}
          <span class="hud__noise-chance"> · {view.noise.visit_chance}%</span>
        </span>
      </div>

      <div class="hud__vitals">
        {view.vitals.map((v) => (
          <Vital key={v.id} v={v} />
        ))}
      </div>

      {view.conditions.length > 0 && (
        <ul class="hud__conditions" aria-label="Conditions">
          {view.conditions.map((c) => (
            <li key={c.id} class="condition">
              <Icon name={c.icon} size={16} />
              <span class="condition__name label">{c.name}</span>
              <span class="condition__detail">{c.detail}</span>
            </li>
          ))}
        </ul>
      )}

      {open && (
        <Sheet title={`Day ${view.day}, ${view.clock}`} onClose={() => setOpen(false)}>
          <p class="prose">{view.location}</p>
          <dl class="table">
            <div class="table__row">
              <dt>Light</dt>
              <dd>{daylight(view)}</dd>
            </div>
            <div class="table__row">
              <dt>Weather</dt>
              <dd>
                {view.weather.today.name}. {view.weather.today.effect}
              </dd>
            </div>
            {view.weather.forecast && (
              <div class="table__row">
                <dt>Forecast</dt>
                <dd>
                  {view.weather.forecast.name}, the radio thinks ({view.weather.forecast_accuracy}%).
                </dd>
              </div>
            )}
            <div class="table__row">
              <dt>Outside</dt>
              <dd class="num">{view.season.temperature}&#176;</dd>
            </div>
            <div class="table__row">
              <dt>The river</dt>
              <dd>
                {view.season.note}
                {view.season.freeze_in !== null ? ` ${view.season.freeze_in} days.` : ''}
              </dd>
            </div>
            <div class="table__row">
              <dt>Noise today</dt>
              <dd>
                {view.noise.value}. {view.noise.label}. {view.noise.visit_chance}% that something comes tonight.
              </dd>
            </div>
          </dl>
          <dl class="table">
            {view.vitals.map((v) => (
              <div key={v.id} class={v.level === 'critical' ? 'table__row table__row--danger' : 'table__row'}>
                <dt>{v.name}</dt>
                <dd class="num">
                  {v.value}
                  {v.level !== 'ok' ? ` ${v.level}` : ''}
                </dd>
              </div>
            ))}
          </dl>
        </Sheet>
      )}
    </header>
  );
}
