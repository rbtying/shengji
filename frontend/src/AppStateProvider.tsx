import * as React from "react";
import { IMessage } from "./ChatMessage";
import gameStatistics, { GameStatistics } from "./state/GameStatistics";
import settings, { Settings } from "./state/Settings";
import { IGameState } from "./types";
import { State, combineState, noPersistence } from "./State";
import { stringLocalStorageState } from "./localStorageState";

export interface AppState {
  settings: Settings;
  gameStatistics: GameStatistics;
  connected: boolean;
  roomName: string;
  name: string;
  game_state: IGameState | null;
  cards: string[];
  errors: string[];
  messages: IMessage[];
}

const appState: State<AppState> = combineState({
  settings,
  gameStatistics,
  connected: noPersistence(() => false),
  roomName: noPersistence(() => window.location.hash.slice(1)),
  name: stringLocalStorageState("name"),
  game_state: noPersistence(() => null),
  cards: noPersistence(() => []),
  errors: noPersistence(() => []),
  messages: noPersistence(() => []),
});

interface Context {
  state: AppState;
  updateState: (newState: Partial<AppState>) => void;
}

export const AppStateContext = React.createContext<Context>({
  state: appState.loadDefault(),
  updateState: () => {},
});

export const SettingsContext = React.createContext<Settings>(
  appState.loadDefault().settings
);

export const AppStateConsumer = AppStateContext.Consumer;

interface IProps {
  children: React.ReactNode;
}
const AppStateProvider = (props: IProps): JSX.Element => {
  const [state, setState] = React.useState<AppState>(() => {
    return appState.loadDefault();
  });
  const updateState = (newState: Partial<AppState>): void => {
    setState((s) => {
      const combined = { ...s, ...newState };
      appState.persist(state, combined);
      return combined;
    });
  };
  return (
    <AppStateContext.Provider value={{ state, updateState }}>
      <SettingsContext.Provider value={state.settings}>
        {props.children}
      </SettingsContext.Provider>
    </AppStateContext.Provider>
  );
};
export default AppStateProvider;
