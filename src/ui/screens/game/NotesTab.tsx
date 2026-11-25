import { useState } from 'preact/hooks';
import type { Action, CodexEntryView, View } from '../../../core';
import { ActionList } from '../../components/ActionButton';
import { Hairline } from '../../components/Hairline';
import { Icon } from '../../components/Icon';
import { Rule } from '../../components/Rule';
import { Sheet } from '../../components/Sheet';

const DOG_STATES: Record<string, string> = {
  unmet: 'Not met',
  following: 'Hanging about',
  companion: 'Yours',
  lost: 'Gone',
  dead: 'Dead',
};

/** One codex entry, read in a sheet. */
export function CodexSheet({ entry, onClose }: { entry: CodexEntryView; onClose: () => void }) {
  return (
    <Sheet title={entry.title} onClose={onClose}>
      <p class="label codex-sheet__kind">
        <Icon name={entry.icon} size={16} /> {KIND_LABEL[entry.kind] ?? entry.kind}
      </p>
      <div class="codex__text prose">
        {entry.text.split('\n').map((para, i) => (
          <p key={i}>{para}</p>
        ))}
      </div>
    </Sheet>
  );
}

const KIND_LABEL: Record<string, string> = {
  note: 'A note',
  photograph: 'A photograph',
  radio: 'Heard on the wireless',
  field_guide: 'Field guide',
};

/** Codex sections with found/total; an entry opens in a sheet; unread ones are marked. */
export function CodexList({
  sections,
  unread,
  onRead,
}: {
  sections: { kind: string; name: string; icon: string; found: number; total: number; entries: CodexEntryView[] }[];
  unread: readonly string[];
  onRead: (id: string) => void;
}) {
  const [open, setOpen] = useState<CodexEntryView | null>(null);
  return (
    <>
      {sections.map((sec) => {
        const fresh = sec.entries.filter((e) => unread.includes(e.id)).length;
        return (
          <section key={sec.kind} class="codex-sec">
            <div class="codex-sec__head">
              <Icon name={sec.icon} size={20} />
              <h3 class="codex-sec__name">{sec.name}</h3>
              {fresh > 0 && <span class="label codex-sec__new">{fresh} new</span>}
              <span class="num codex-sec__count">
                {sec.found}/{sec.total}
              </span>
            </div>
            <Hairline value={sec.found} max={sec.total} label={`${sec.name}: ${sec.found} of ${sec.total} found`} />
            {sec.entries.length === 0 ? (
              <p class="codex-sec__empty">Nothing yet.</p>
            ) : (
              <ul class="codex-sec__list">
                {sec.entries.map((e) => {
                  const isNew = unread.includes(e.id);
                  return (
                    <li key={e.id}>
                      <button
                        type="button"
                        class={isNew ? 'codex-entry codex-entry--new' : 'codex-entry'}
                        onClick={() => {
                          setOpen(e);
                          onRead(e.id);
                        }}
                      >
                        <span class="codex-entry__title">{e.title}</span>
                        {isNew && <span class="label codex-entry__new">New</span>}
                        <Icon name="forward" size={16} />
                      </button>
                    </li>
                  );
                })}
              </ul>
            )}
          </section>
        );
      })}
      {open && <CodexSheet entry={open} onClose={() => setOpen(null)} />}
    </>
  );
}

export function NotesTab({
  view,
  onAct,
  unread,
  onRead,
}: {
  view: View;
  onAct: (a: Action) => void;
  unread: readonly string[];
  onRead: (id: string) => void;
}) {
  const ferry = view.ferry;
  const ferryActs = view.actions.filter((a) => a.group === 'ferry');
  const dog = view.dog;
  return (
    <>
      <section class="section">
        <Rule>The ferry</Rule>
        <div class="figure-line">
          <span class="label">
            <Icon name="ferry" size={16} /> The Kittiwake, parts fitted
          </span>
          <span class="num">
            {ferry.installed}/{ferry.parts.length}
          </span>
        </div>
        <Hairline value={ferry.installed} max={ferry.parts.length} steps label="Ferry parts fitted" />
        <ul class="parts">
          {ferry.parts.map((p) => (
            <li key={p.id} class={p.installed ? 'part part--in' : p.in_store ? 'part part--held' : 'part'}>
              <Icon name={p.installed ? 'check' : p.icon} size={16} />
              <span class="part__body">
                <span class="part__name">{p.name}</span>
                {p.need && <span class="part__need">{p.need}</span>}
              </span>
              <span class="part__state label">{p.installed ? 'Fitted' : p.in_store ? 'In the store' : 'Not found'}</span>
            </li>
          ))}
        </ul>
        <div class="voyage">
          <p class="label voyage__head">For the voyage, two days to the estuary</p>
          <div class="figure-line">
            <span class="label">
              <Icon name="food" size={16} /> Meals put by
            </span>
            <span class="num">
              {Math.min(ferry.meals, ferry.meals_need)}/{ferry.meals_need}
            </span>
          </div>
          <Hairline value={ferry.meals} max={ferry.meals_need} label="Meals put by for the voyage" />
          <div class="figure-line">
            <span class="label">
              <Icon name="water" size={16} /> Clean water put by
            </span>
            <span class="num">
              {Math.min(ferry.water, ferry.water_need)}/{ferry.water_need}
            </span>
          </div>
          <Hairline value={ferry.water} max={ferry.water_need} label="Clean water put by for the voyage" />
        </div>
        {ferry.frozen && <p class="prose danger-text">The river is frozen. The ferry will not go now.</p>}
        {ferryActs.length > 0 && <ActionList actions={ferryActs} onAct={onAct} />}
      </section>

      {dog && dog.state !== 'unmet' && (
        <section class="section">
          <Rule>The dog</Rule>
          <p class="prose">{dog.line}</p>
          <div class="figure-line">
            <span class="label">
              <Icon name="sigil_dog" size={16} /> {DOG_STATES[dog.state] ?? dog.state}
            </span>
            <span class="num">{dog.with_you ? 'with you' : ''}</span>
          </div>
          {(dog.state === 'companion' || dog.state === 'following') && (
            <>
              <div class="figure-line">
                <span class="label">Health</span>
                <span class="num">{dog.health}</span>
              </div>
              <Hairline value={dog.health} max={100} label="The dog's health" danger={dog.health < 15} />
              <div class="figure-line">
                <span class="label">Fed</span>
                <span class="num">{dog.fed}</span>
              </div>
              <Hairline value={dog.fed} max={100} label="How well fed the dog is" danger={dog.fed <= 0} />
            </>
          )}
        </section>
      )}

      <section class="section">
        <Rule>What you have found</Rule>
        <div class="figure-line">
          <span class="label">The codex</span>
          <span class="num">
            {view.codex.found}/{view.codex.total}
          </span>
        </div>
        <CodexList sections={view.codex.sections} unread={unread} onRead={onRead} />
      </section>

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
              <Hairline value={s.level} max={5} steps label={`${s.name} level ${s.level} of 5`} />
              {s.next !== null && (
                <Hairline value={s.xp} max={s.next} label={`${s.name}, practice toward the next level`} />
              )}
              <p class="skill__effect">{s.effect}</p>
            </li>
          ))}
        </ul>
      </section>

      {view.marks.length > 0 && (
        <section class="section">
          <Rule>Marks this run</Rule>
          <ul class="marks">
            {view.marks.map((m) => (
              <li key={m.id} class="mark mark--earned">
                <Icon name={m.icon} size={24} class="mark__sigil" />
                <div class="mark__body">
                  <p class="mark__name">{m.name}</p>
                  <p class="mark__text">{m.text}</p>
                </div>
              </li>
            ))}
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
