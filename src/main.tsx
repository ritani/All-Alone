import { render } from 'preact';
import './ui/styles/tokens.css';
import './ui/styles/base.css';
import './ui/styles/components.css';
import './ui/styles/screens.css';
import { App } from './ui/App';
import { initCore } from './core';

const root = document.getElementById('app');
if (!root) throw new Error('#app root missing');
render(<App />, root);

// Load the game core early; the screens that need it arrive in M3.
initCore().catch((e: unknown) => {
  console.error('The game core failed to load.', e);
});
