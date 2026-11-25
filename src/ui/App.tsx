import { Kit } from './screens/Kit';
import { Title } from './screens/Title';

/** Temporary review route: `?kit` shows the primitives sheet. */
function wantsKit(): boolean {
  return new URLSearchParams(window.location.search).has('kit');
}

export function App() {
  return wantsKit() ? <Kit /> : <Title />;
}
