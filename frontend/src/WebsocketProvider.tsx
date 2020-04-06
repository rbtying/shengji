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

  React.useEffect(() => {
    const uri =
      (location.protocol === 'https:' ? 'wss://' : 'ws://') +
      location.host +
      location.pathname +
      (location.pathname.endsWith('/') ? 'api' : '/api');

    const ws = new WebSocket(uri);
    setWebsocket(ws);

    ws.addEventListener('open', () => updateState({connected: true}));
    ws.addEventListener('close', () => updateState({connected: false}));
    ws.addEventListener('message', (event: MessageEvent) => {
      const message = JSON.parse(event.data);
      if (message === 'Kicked') {
        ws.close();
      } else {
        updateState(websocketHandler(state, message));
      }
    });
  }, []);

  const send = (value: any) => websocket?.send(JSON.stringify(value));

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
