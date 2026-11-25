import { useState } from 'preact/hooks';
import type { Action, ProfileView, View } from '../../../core';
import { ActionList } from '../../components/ActionButton';
import { Hairline } from '../../components/Hairline';
import { Icon } from '../../components/Icon';
import { Rule } from '../../components/Rule';

const DOG_STATES: Record<string, string> = {
  unmet: 'Not met',
  following: 'Following',
  companion: 'With you',
  lost: 'Gone',
  dead: 'Dead',
};

export function NotesTab({ view, profile, onAct }: { view: View; profile: ProfileView | null; onAct: (a: Action) => void }) {
  const [openSection, setOpenSection] = useState<string | null>(null);
  const [openEntry, setOpenEntry] = useState<string | null>(null);
  const ferryActs = view.actions.filter((a) => a.group === 'ferry');
  const markInfo = new Map((profile?.marks ?? []).map((m) => [m.id, m]));
  return (
    <>
      <section class="section">
        <Rule>The ferry</Rule>
        <div class="figure-line">
          <span class="label">
            <Icon name="ferry" size={16} /> Parts fitted
          </span>
          <span class="num">
            {view.ferry.installed}/{view.ferry.parts.length}
          </span>
        </div>
        <Hairline value={view.ferry.installed} max={view.ferry.parts.length} steps label="Ferry parts fitted" />
        <ul class="parts">
          {view.ferry.parts.map((p) => (
            <li key={p.id} class={p.installed ? 'part part--in' : p.in_store ? 'part part--held' : 'part'}>
              <Icon name={p.installed ? 'check' : p.icon} size={16} />
              <span class="part__name">{p.name}</span>
              <span class="part__state label">{p.installed ? 'Fitted' : p.in_store ? 'In the store' : 'Not found'}</span>
            </li>
          ))}
        </ul>
        {view.ferry.frozen && <p class="prose danger-text">The river is frozen. The ferry will not go now.</p>}
        {ferryActs.length > 0 && <ActionList actions={ferryActs} onAct={onAct} />}
      </section>

      {view.dog && view.dog.state !== 'unmet' && (
        <section class="section">
          <Rule>The dog</Rule>
          <p class="prose">{view.dog.line}</p>
          <div class="figure-line">
            <span class="label">
              <Icon name="sigil_dog" size={16} /> {DOG_STATES[view.dog.state] ?? view.dog.state}
            </span>
            <span class="num">health {view.dog.health} · fed {view.dog.fed}</span>
          </div>
          <Hairline value={view.dog.health} max={100} label="The dog's health" danger={view.dog.health < 15} />
        </section>
      )}

      <section class="section">
        <Rule>Skills</Rule>
        <ul class="skills">
          {view.skills.map((s) => (
            <li key={s.id} class="skill">
              <div class="figure-line">
                <span class="label">
                  <Icon name={s.icon} size={16} /> {s.name}
                </span>
                <span class="num">{s.level}/5</span>
              </div>
              <Hairline value={s.level} max={5} steps label={`${s.name} level`} />
              {s.next !== null && (
                <Hairline value={s.xp} max={s.next} label={`${s.name}, practice toward the next level`} />
              )}
              <p class="skill__effect">{s.effect}</p>
            </li>
          ))}
        </ul>
      </section>

      <section class="section">
        <Rule>Codex</Rule>
        <div class="figure-line">
          <span class="label">Found</span>
          <span class="num">
            {view.codex.found}/{view.codex.total}
          </span>
        </div>
        <Hairline value={view.codex.found} max={view.codex.total} label="Codex found" />
        <ul class="codex">
          {view.codex.sections.map((sec) => {
            const open = openSection === sec.kind;
            return (
              <li key={sec.kind} class="codex__section">
                <button
                  type="button"
                  class="disclosure"
                  aria-expanded={open}
                  disabled={sec.entries.length === 0}
                  onClick={() => setOpenSection(open ? null : sec.kind)}
                >
                  <Icon name={sec.icon} size={20} />
                  <span class="disclosure__name">{sec.name}</span>
                  <span class="num">
                    {sec.found}/{sec.total}
                  </span>
                  <Icon name="down" size={16} class={open ? 'disclosure__icon disclosure__icon--open' : 'disclosure__icon'} />
                </button>
                {open && (
                  <ul class="codex__entries">
                    {sec.entries.map((e) => {
                      const on = openEntry === e.id;
                      return (
                        <li key={e.id} class="codex__entry">
                          <button type="button" class="codex__title" aria-expanded={on} onClick={() => setOpenEntry(on ? null : e.id)}>
                            {e.title}
                          </button>
                          {on && (
                            <div class="codex__text prose">
                              {e.text.split('\n').map((para, i) => (
                                <p key={i}>{para}</p>
                              ))}
                            </div>
                          )}
                        </li>
                      );
                    })}
                  </ul>
                )}
              </li>
            );
          })}
        </ul>
      </section>

      {view.marks.length > 0 && (
        <section class="section">
          <Rule>Marks this run</Rule>
          <ul class="marks">
            {view.marks.map((id) => {
              const m = markInfo.get(id);
              const known = m && !(m.hidden && !m.earned);
              return (
                <li key={id} class="mark mark--earned">
                  <Icon name={m?.icon ?? 'sigil_eye'} size={24} class="mark__sigil" />
                  <div class="mark__body">
                    <p class="mark__name">{known ? m.name : 'A hidden mark'}</p>
                    {known && <p class="mark__text">{m.text}</p>}
                  </div>
                </li>
              );
            })}
          </ul>
        </section>
      )}

      <section class="section">
        <Rule>This run</Rule>
        <dl class="table">
          <div class="table__row">
            <dt>Who you were</dt>
            <dd>{view.run.background_name}</dd>
          </div>
          <div class="table__row">
            <dt>How hard</dt>
            <dd>{view.run.difficulty_name}</dd>
          </div>
          {view.run.daily && (
            <div class="table__row">
              <dt>Daily river</dt>
              <dd class="num">{view.run.daily}</dd>
            </div>
          )}
          <div class="table__row">
            <dt>Nights survived</dt>
            <dd class="num">{view.run.nights}</dd>
          </div>
          <div class="table__row">
            <dt>Killed</dt>
            <dd class="num">{view.run.kills}</dd>
          </div>
          <div class="table__row">
            <dt>Seed</dt>
            <dd class="num">{view.run.seed}</dd>
          </div>
        </dl>
      </section>
    </>
  );
}
