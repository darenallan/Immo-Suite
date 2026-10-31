import { render, screen } from '@testing-library/react';
import { App } from './App';
import { fr } from './texts/fr';

describe('page technique', () => {
  it('affiche le nom du produit et la page technique', () => {
    render(<App />);
    expect(screen.getByRole('heading', { level: 1, name: fr.app.name })).toBeInTheDocument();
    expect(screen.getByRole('heading', { level: 2, name: fr.home.title })).toBeInTheDocument();
  });

  it('affiche la version et l’environnement injectés à la compilation', () => {
    render(<App />);
    expect(screen.getByTestId('version')).toHaveTextContent(/^\S+$/);
    expect(screen.getByTestId('environment')).toHaveTextContent(/^\S+$/);
  });
});
