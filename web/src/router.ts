// Routage minimal par l'historique du navigateur (aucune bibliothèque : le budget de poids est de 150 Ko de JavaScript).
import { useCallback, useEffect, useState } from 'react';

export function currentPath(): string {
  return window.location.pathname;
}

export function usePath(): [string, (path: string) => void] {
  const [path, setPath] = useState(currentPath());
  useEffect(() => {
    const onPop = () => {
      setPath(currentPath());
    };
    window.addEventListener('popstate', onPop);
    return () => {
      window.removeEventListener('popstate', onPop);
    };
  }, []);
  const navigate = useCallback((to: string) => {
    if (to !== currentPath()) window.history.pushState(null, '', to);
    setPath(to);
  }, []);
  return [path, navigate];
}
