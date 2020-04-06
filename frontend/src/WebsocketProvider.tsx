import * as React from 'react';
import {AppState, AppStateConsumer} from './AppStateProvider';
import websocketHandler from './websocketHandler';

type Context = {
  send: (value: any) => void;
};

const WebsocketContext = React.createContext<Context>({
  send: () => {},
});

export const WebsocketConsumer = WebsocketContext.Consumer;

type Props = {
  state: AppState;
  updateState: (state: Partial<AppState>) => void;
  children?: React.ReactNode;
};
const WebsocketProvider = (props: Props) => {
  const {state, updateState, children} = props;
  const [websocket, setWebsocket] = React.useState<WebSocket | null>(null);

  // Because state/updateState are passed in and change every time something
  // happens, we need to maintain a reference to these props to prevent stale
  // closures which may happen if state/updateState is changed between when an
  // event listener is registered and when it fires.
  // https://reactjs.org/docs/hooks-faq.html#why-am-i-seeing-stale-props-or-state-inside-my-function
  const stateRef = React.useRef(state);
  const updateStateRef = React.useRef(updateState);

  React.useEffect(() => {
    stateRef.current = state;
    updateStateRef.current = updateState;
  }, [state, updateState]);

  React.useEffect(() => {
    const uri =
      (location.protocol === 'https:' ? 'wss://' : 'ws://') +
      location.host +
      location.pathname +
      (location.pathname.endsWith('/') ? 'api' : '/api');

    const ws = new WebSocket(uri);
    setWebsocket(ws);

    ws.addEventListener('open', () =>
      updateStateRef.current({connected: true}),
    );
    ws.addEventListener('close', () =>
      updateStateRef.current({connected: false}),
    );
    ws.addEventListener('message', (event: MessageEvent) => {
      const message = JSON.parse(event.data);
      if (message === 'Kicked') {
        ws.close();
      } else {
        updateStateRef.current(websocketHandler(stateRef.current, message));
      }
    });
  }, []);

  const send = (value: any) => websocket?.send(JSON.stringify(value));
  // TODO(read this from consumers instead of globals)
  (window as any).send = send;

  return (
    <WebsocketContext.Provider value={{send}}>
      {children}
    </WebsocketContext.Provider>
  );
};

export default () => {
  return (
    <AppStateConsumer>
      {({state, updateState}) => (
        <WebsocketProvider state={state} updateState={updateState} />
      )}
    </AppStateConsumer>
  );
};
