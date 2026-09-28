import * as React from "react";
import { AppStateContext, AppState } from "./AppStateProvider";
import { WebsocketContext } from "./WebsocketProvider";
import websocketHandler from "./websocketHandler";
import { GameMessage } from "./gen-types";

import type { JSX } from "react";

// The room name used for offline play. It has to be 16 characters, like a real room code.
export const OFFLINE_ROOM = "offline-practice";
const SNAPSHOT_KEY = "offline_game_snapshot";

export const isOffline = (): boolean =>
  window.location.hash.slice(1, 17) === OFFLINE_ROOM;

/** Forget the saved offline game so the next visit starts a fresh table. */
export const clearOfflineGame = (): void => {
  try {
    window.localStorage.removeItem(SNAPSHOT_KEY);
  } catch {
    // Nothing saved, or storage unavailable.
  }
};

interface IProps {
  children: JSX.Element[] | JSX.Element;
}

/**
 * Stands in for WebsocketProvider: the game engine and computer players run in a web worker
 * instead of on a server, speaking the same messages.
 */
const OfflineProvider = (props: IProps): JSX.Element => {
  const { state, updateState } = React.useContext(AppStateContext);
  const stateRef = React.useRef<AppState>(state);
  const updateStateRef = React.useRef(updateState);
  const workerRef = React.useRef<Worker | null>(null);

  React.useEffect(() => {
    stateRef.current = state;
    updateStateRef.current = updateState;
  }, [state, updateState]);

  const send = React.useCallback((value: any): void => {
    workerRef.current?.postMessage({ type: "send", payload: value });
  }, []);

  React.useEffect(() => {
    const worker = new Worker(new URL("./offlineWorker.ts", import.meta.url));
    workerRef.current = worker;
    worker.onmessage = (e: MessageEvent) => {
      const data = e.data;
      if (data.type === "snapshot") {
        try {
          window.localStorage.setItem(SNAPSHOT_KEY, data.snapshot);
        } catch {
          // Saving is best-effort.
        }
      } else if (data.type === "messages") {
        let current = stateRef.current;
        let partial: Partial<AppState> = {};
        (data.messages as GameMessage[]).forEach((m) => {
          const update = websocketHandler(current, m, send);
          partial = { ...partial, ...update };
          current = { ...current, ...update };
        });
        stateRef.current = current;
        updateStateRef.current(partial);
      }
    };
    let snapshot: string | null = null;
    try {
      snapshot = window.localStorage.getItem(SNAPSHOT_KEY);
    } catch {
      // Start fresh.
    }
    worker.postMessage({ type: "init", snapshot });
    updateStateRef.current({ connected: true, everConnected: true });
    return () => worker.terminate();
  }, []);

  // TODO(read this from consumers instead of globals)
  (window as any).send = send;

  return (
    <WebsocketContext.Provider value={{ send }}>
      {props.children}
    </WebsocketContext.Provider>
  );
};

export default OfflineProvider;
