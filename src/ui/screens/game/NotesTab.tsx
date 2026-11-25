import { useState } from 'preact/hooks';
import type { Action, CodexEntryView, View } from '../../../core';
import { ActionList } from '../../components/ActionButton';
import { Hairline } from '../../components/Hairline';
import { Icon } from '../../components/Icon';
import { ListRow } from '../../components/ListRow';
import { Rule } from '../../components/Rule';
import { Sheet } from '../../components/Sheet';

const DOG_STATES: Record<string, string> = {
  unmet: 'Not met',
  following: 'Hanging about',
  companion: 'Yours',
  lost: 'Gone',
  dead: 'Dead',
};

const KIND_LABEL: Record<string, string> = {
  note: 'A note',
  photograph: 'A photograph',
  radio: 'Heard on the wireless',
  field_guide: 'Field guide',
};

function CodexText({ entry }: { entry: CodexEntryView }) {
  return (
    <>
      <p class="codex-sheet__kind">
        <Icon name={entry.icon} size={16} /> {KIND_LABEL[entry.kind] ?? entry.kind}
      </p>
      <div class="codex__text prose">
        {entry.text.split('\n').map((para, i) => (
          <p key={i}>{para}</p>
        ))}
      </div>
    </>
  );
}

/** One codex entry, read in a sheet. */
export function CodexSheet({ entry, onClose }: { entry: CodexEntryView; onClose: () => void }) {
  return (
    <Sheet title={entry.title} onClose={onClose}>
      <CodexText entry={entry} />
    </Sheet>
  );
}

type Section = { kind: string; name: string; icon: string; found: number; total: number; entries: CodexEntryView[] };

function EntryRows({ entries, unread, onPick }: { entries: CodexEntryView[]; unread: readonly string[]; onPick: (e: CodexEntryView) => void }) {
  if (entries.length === 0) return <p class="prose prose--dim">Nothing yet.</p>;
  return (
    <ul class="lrows">
      {entries.map((e) => {
        const isNew = unread.includes(e.id);
        return (
          <li key={e.id}>
            <ListRow title={e.title} flag={isNew ? 'Not read' : undefined} dim={!isNew} onClick={() => onPick(e)} />
          </li>
        );
      })}
    </ul>
  );
}

/**
 * Codex sections as rows with found/total. A section opens a sheet listing
 * its entries; an entry is read in the same sheet, with a way back.
 */
export function CodexList({ sections, unread, onRead }: { sections: Section[]; unread: readonly string[]; onRead: (id: string) => void }) {
  const [sec, setSec] = useState<string | null>(null);
  const [entry, setEntry] = useState<CodexEntryView | null>(null);
  const open = sections.find((s) => s.kind === sec);
  const close = () => {
    setSec(null);
    setEntry(null);
  };
  return (
    <>
      <ul class="lrows">
        {sections.map((s) => {
          const fresh = s.entries.filter((e) => unread.includes(e.id)).length;
          return (
            <li key={s.kind}>
              <ListRow
                icon={s.icon}
                title={s.name}
                flag={fresh > 0 ? `${fresh} not read` : undefined}
                aside={
                  <span class="num">
                    {s.found}/{s.total}
                  </span>
                }
                onClick={() => setSec(s.kind)}
              />
            </li>
          );
        })}
      </ul>
      {open && (
        <Sheet
          title={entry ? entry.title : open.name}
          tall={entry !== null}
          onClose={close}
          onBack={entry ? () => setEntry(null) : undefined}
          lead={
            entry ? (
              <button type="button" class="sheet__back" aria-label={`Back to ${open.name}`} onClick={() => setEntry(null)}>
                <Icon name="back" size={20} />
              </button>
            ) : undefined
          }
        >
          {entry ? (
            <CodexText entry={entry} />
          ) : (
            <>
              <Hairline value={open.found} max={open.total} label={`${open.name}: ${open.found} of ${open.total} found`} />
              <EntryRows
                entries={open.entries}
                unread={unread}
                onPick={(e) => {
                  setEntry(e);
                  onRead(e.id);
                }}
              />
            </>
          )}
        </Sheet>
      )}
    </>
  );
}

type Page = 'ferry' | 'dog' | 'skills' | 'marks' | 'run';

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
  const [page, setPage] = useState<Page | null>(null);
  const ferry = view.ferry;
  const ferryActs = view.actions.filter((a) => a.group === 'ferry');
  const dog = view.dog;
  const close = () => setPage(null);
  const skillSum = view.skills.reduce((n, s) => n + s.level, 0);
  return (
    <>
      <section class="section">
        <ul class="lrows">
          <li>
            <ListRow
              icon="ferry"
              title="The Kittiwake"
              sub={`Meals ${Math.min(ferry.meals, ferry.meals_need)}/${ferry.meals_need} · water ${Math.min(ferry.water, ferry.water_need)}/${ferry.water_need}`}
              aside={
                <span class="num">
                  {ferry.installed}/{ferry.parts.length} fitted
                </span>
              }
              danger={ferry.frozen}
              flag={ferryActs.some((a) => a.enabled) ? 'Something to do' : undefined}
              onClick={() => setPage('ferry')}
            />
          </li>
          {dog && dog.state !== 'unmet' && (
            <li>
              <ListRow icon="sigil_dog" title="The dog" aside={DOG_STATES[dog.state] ?? dog.state} onClick={() => setPage('dog')} />
            </li>
          )}
          <li>
            <ListRow icon="skill_quiet" title="Skills" aside={<span class="num">{skillSum}</span>} onClick={() => setPage('skills')} />
          </li>
          {view.marks.length > 0 && (
            <li>
              <ListRow icon="sigil_moon" title="Marks this run" aside={<span class="num">{view.marks.length}</span>} onClick={() => setPage('marks')} />
            </li>
          )}
          <li>
            <ListRow icon="journal" title="This run" aside={<span class="num">{view.run.nights} nights</span>} onClick={() => setPage('run')} />
          </li>
        </ul>
      </section>

      <section class="section">
        <Rule>
          Found{' '}
          <span class="num rule__count">
            {view.codex.found}/{view.codex.total}
          </span>
        </Rule>
        <CodexList sections={view.codex.sections} unread={unread} onRead={onRead} />
      </section>

      {page === 'ferry' && (
        <Sheet title="The Kittiwake" onClose={close}>
          <Hairline value={ferry.installed} max={ferry.parts.length} steps label="Ferry parts fitted" />
          <ul class="parts">
            {ferry.parts.map((p) => (
              <li key={p.id} class={p.installed ? 'part part--in' : p.in_store ? 'part part--held' : 'part'}>
                <Icon name={p.installed ? 'check' : p.icon} size={20} />
                <span class="part__body">
                  <span class="part__name">{p.name}</span>
                  {p.need && <span class="part__need">{p.need}</span>}
                </span>
                <span class="part__state">{p.installed ? 'Fitted' : p.in_store ? 'In the store' : 'Not found'}</span>
              </li>
            ))}
          </ul>
          <Rule level={3}>For the voyage, two days</Rule>
          <div class="figure">
            <span class="figure__name">
              <Icon name="food" size={16} /> Meals put by
            </span>
            <span class="num">
              {Math.min(ferry.meals, ferry.meals_need)}/{ferry.meals_need}
            </span>
            <Hairline value={ferry.meals} max={ferry.meals_need} label="Meals put by for the voyage" />
          </div>
          <div class="figure">
            <span class="figure__name">
              <Icon name="water" size={16} /> Clean water put by
            </span>
            <span class="num">
              {Math.min(ferry.water, ferry.water_need)}/{ferry.water_need}
            </span>
            <Hairline value={ferry.water} max={ferry.water_need} label="Clean water put by for the voyage" />
          </div>
          {ferry.frozen && <p class="prose danger-text">The river is frozen. The ferry will not go now.</p>}
          {ferryActs.length > 0 && <ActionList actions={ferryActs} onAct={onAct} />}
        </Sheet>
      )}

      {page === 'dog' && dog && (
        <Sheet title="The dog" onClose={close}>
          <p class="prose">{dog.line}</p>
          <p class="sheet__state">
            {DOG_STATES[dog.state] ?? dog.state}
            {dog.with_you ? ', with you' : ''}
          </p>
          {(dog.state === 'companion' || dog.state === 'following') && (
            <>
              <div class="figure">
                <span class="figure__name">Health</span>
                <span class="num">{dog.health}</span>
                <Hairline value={dog.health} max={100} label="The dog's health" danger={dog.health < 15} />
              </div>
              <div class="figure">
                <span class="figure__name">Fed</span>
                <span class="num">{dog.fed}</span>
                <Hairline value={dog.fed} max={100} label="How well fed the dog is" danger={dog.fed <= 0} />
              </div>
            </>
          )}
        </Sheet>
      )}

      {page === 'skills' && (
        <Sheet title="Skills" onClose={close}>
          <ul class="skills">
            {view.skills.map((s) => (
              <li key={s.id} class="skill">
                <div class="figure">
                  <span class="figure__name">
                    <Icon name={s.icon} size={16} /> {s.name}
                  </span>
                  <span class="num">{s.level}/5</span>
                  <Hairline value={s.level} max={5} steps label={`${s.name} level ${s.level} of 5`} />
                </div>
                {s.next !== null && <Hairline value={s.xp} max={s.next} label={`${s.name}, practice toward the next level`} />}
                <p class="skill__effect">{s.effect}</p>
              </li>
            ))}
          </ul>
        </Sheet>
      )}

      {page === 'marks' && (
        <Sheet title="Marks this run" onClose={close}>
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
        </Sheet>
      )}

      {page === 'run' && (
        <Sheet title="This run" onClose={close}>
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
        </Sheet>
      )}
    </>
  );
}
