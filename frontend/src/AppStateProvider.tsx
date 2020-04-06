import * as React from 'react';
import {Message} from './Chat';
import settings, {Settings} from './state/Settings';
import {IGameState} from './types';
import {State, combineState, noPersistence} from './State';
import {stringLocalStorageState} from './localStorageState';

export type AppState = {
  settings: Settings;
  connected: boolean;
  roomName: string;
  name: string;
  game_state: IGameState | null;
  cards: string[];
  errors: string[];
  messages: Message[];
};

const appState: State<AppState> = combineState({
  settings,
  connected: noPersistence(() => false),
  roomName: noPersistence(() => window.location.hash.slice(1)),
  name: stringLocalStorageState('name'),
  game_state: noPersistence(() => null),
  cards: noPersistence(() => []),
  errors: noPersistence(() => []),
  messages: noPersistence(() => []),
});

type Context = {
  state: AppState;
  updateState: (newState: Partial<AppState>) => void;
};

const AppStateContext = React.createContext<Context>({
  state: appState.loadDefault(),
  updateState: () => {},
});

export const AppStateConsumer = AppStateContext.Consumer;

type Props = {
  children: React.ReactNode;
};
const AppStateProvider = (props: Props) => {
  const [state, setState] = React.useState<AppState>(() => {
    return appState.loadDefault();
  });
  const updateState = (newState: Partial<AppState>) => {
    const combined = {...state, ...newState};
    appState.persist(state, combined);
    setState(combined);
  };
  return (
    <AppStateContext.Provider value={{state, updateState}}>
      {props.children}
    </AppStateContext.Provider>
  );
};
export default AppStateProvider;
