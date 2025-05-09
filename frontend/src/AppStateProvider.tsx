import * as React from "react";
import gameStatistics, { GameStatistics } from "./state/GameStatistics";
import settings, { Settings } from "./state/Settings";
import { GameState } from "./gen-types";
import { Message } from "./ChatMessage";
import { State, combineState, noPersistence } from "./State";
import {
  stringLocalStorageState,
  numberLocalStorageState,
} from "./localStorageState";

import type { JSX } from "react";

export interface AppState {
  settings: Settings;
  gameStatistics: GameStatistics;
  connected: boolean;
  everConnected: boolean;
  roomName: string;
  name: string;
  gameState: GameState | null;
  headerMessages: string[];
  errors: string[];
  messages: Message[];
  confetti: string | null;
  changeLogLastViewed: number;
}

const appState: State<AppState> = combineState({
  settings,
  gameStatistics,
  connected: noPersistence(() => false),
  everConnected: noPersistence(() => false),
  roomName: noPersistence(() => window.location.hash.slice(1, 17)),
  name: stringLocalStorageState("name"),
  changeLogLastViewed: numberLocalStorageState("change_log_last_viewed"),
  gameState: noPersistence<GameState | null>(() => null),
  headerMessages: noPersistence<string[]>(() => []),
  errors: noPersistence<string[]>(() => []),
  messages: noPersistence<Message[]>(() => []),
  confetti: noPersistence<string | null>(() => null),
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
  appState.loadDefault().settings,
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
