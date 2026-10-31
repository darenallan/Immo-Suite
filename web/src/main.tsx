import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import './styles/tokens.css';
import './styles/app.css';

const root = document.getElementById('root');
if (root === null) {
  throw new Error('Élément #root introuvable');
}
createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
