import * as React from "react";
import { AppStateContext } from "./AppStateProvider";
import websocketHandler from "./websocketHandler";
import { TimerContext } from "./TimerProvider";

interface Context {
  send: (value: any) => void;
}

export const WebsocketContext = React.createContext<Context>({
  send: () => {},
});

interface IProps {
  children: JSX.Element[] | JSX.Element;
}

const WebsocketProvider: React.FunctionComponent<IProps> = (props: IProps) => {
  const { state, updateState } = React.useContext(AppStateContext);
  const { setTimeout, clearTimeout } = React.useContext(TimerContext);
  const [timer, setTimer] = React.useState<number | null>(null);
  const [websocket, setWebsocket] = React.useState<WebSocket | null>(null);

  // Because state/updateState are passed in and change every time something
  // happens, we need to maintain a reference to these props to prevent stale
  // closures which may happen if state/updateState is changed between when an
  // event listener is registered and when it fires.
  // https://reactjs.org/docs/hooks-faq.html#why-am-i-seeing-stale-props-or-state-inside-my-function
  const stateRef = React.useRef(state);
  const updateStateRef = React.useRef(updateState);
  const timerRef = React.useRef(timer);
  const setTimerRef = React.useRef(setTimer);
  const setTimeoutRef = React.useRef(setTimeout);
  const clearTimeoutRef = React.useRef(clearTimeout);

  React.useEffect(() => {
    stateRef.current = state;
    updateStateRef.current = updateState;
  }, [state, updateState]);

  React.useEffect(() => {
    setTimeoutRef.current = setTimeout;
    clearTimeoutRef.current = clearTimeout;
  }, [setTimeout, clearTimeout]);

  React.useEffect(() => {
    timerRef.current = timer;
    setTimerRef.current = setTimer;
  }, [timer, setTimerRef]);

  React.useEffect(() => {
    const uri =
      (location.protocol === "https:" ? "wss://" : "ws://") +
      location.host +
      location.pathname +
      (location.pathname.endsWith("/") ? "api" : "/api");

    const ws = new WebSocket(uri);
    setWebsocket(ws);

    ws.addEventListener("open", () =>
      updateStateRef.current({ connected: true })
    );
    ws.addEventListener("close", () =>
      updateStateRef.current({ connected: false })
    );
    ws.addEventListener("message", (event: MessageEvent) => {
      if (timerRef.current !== null) {
        clearTimeoutRef.current(timerRef.current);
      }
      setTimerRef.current(null);

      const message = JSON.parse(event.data);
      if (message === "Kicked") {
        ws.close();
      } else {
        updateStateRef.current({
          connected: true,
          ...websocketHandler(stateRef.current, message),
        });
      }
    });

    return () => {
      if (timerRef.current !== null) {
        clearTimeoutRef.current(timerRef.current);
      }
    };
  }, []);

  const send = (value: any): void => {
    if (timerRef.current !== null) {
      clearTimeoutRef.current(timerRef.current);
    }
    // We expect a response back from the server within 5 seconds. Otherwise,
    // we should assume we have lost our websocket connection.

    const localTimerRef = setTimeoutRef.current(() => {
      if (timerRef.current === localTimerRef) {
        updateStateRef.current({ connected: false });
      }
    }, 5000);

    setTimerRef.current(localTimerRef);
    websocket?.send(JSON.stringify(value));
  };
  // TODO(read this from consumers instead of globals)
  (window as any).send = send;

  return (
    <WebsocketContext.Provider value={{ send }}>
      {props.children}
    </WebsocketContext.Provider>
  );
};

export default WebsocketProvider;
