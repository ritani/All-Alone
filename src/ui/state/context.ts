import { createContext } from 'preact';
import { useContext, useEffect, useState } from 'preact/hooks';
import type { AppState, Store } from './store';

export const StoreContext = createContext<Store | null>(null);

export function useStoreInstance(): Store {
  const store = useContext(StoreContext);
  if (!store) throw new Error('StoreContext missing');
  return store;
}

/** The current app state; re-renders on every change. */
export function useAppState(): AppState {
  const store = useStoreInstance();
  const [state, setState] = useState(store.get());
  useEffect(() => {
    setState(store.get());
    return store.subscribe(setState);
  }, [store]);
  return state;
}
