import { render } from 'preact';
import './ui/styles/tokens.css';
import './ui/styles/base.css';
import './ui/styles/components.css';
import './ui/styles/screens.css';
import { App } from './ui/App';

const root = document.getElementById('app');
if (!root) throw new Error('#app root missing');
render(<App />, root);
