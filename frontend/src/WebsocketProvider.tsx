import * as React from 'react';
import { AppStateContext } from './AppStateProvider';
import websocketHandler from './websocketHandler';
import { TimerContext } from './TimerProvider';

type Context = {
  send: (value: any) => void;
};

export const WebsocketContext = React.createContext<Context>({
  send: () => { },
});

const WebsocketProvider: React.FunctionComponent<{}> = (props) => {
  const { state, updateState } = React.useContext(AppStateContext);
  const { setTimeout, clearTimeout } = React.useContext(TimerContext);
  // const [timer, setTimer] = React.useState<number | null>(null);
  const [websocket, setWebsocket] = React.useState<WebSocket | null>(null);

  // Because state/updateState are passed in and change every time something
  // happens, we need to maintain a reference to these props to prevent stale
  // closures which may happen if state/updateState is changed between when an
  // event listener is registered and when it fires.
  // https://reactjs.org/docs/hooks-faq.html#why-am-i-seeing-stale-props-or-state-inside-my-function
  const stateRef = React.useRef(state);
  const updateStateRef = React.useRef(updateState);
  // const timerRef = React.useRef(timer);
  // const setTimerRef = React.useRef(setTimer);
  const setTimeoutRef = React.useRef(setTimeout);
  const clearTimeoutRef = React.useRef(clearTimeout);

  const [timerList, setTimerList] = React.useState([]);

  React.useEffect(() => {
    stateRef.current = state;
    updateStateRef.current = updateState;
  }, [state, updateState]);

  React.useEffect(() => {
    setTimeoutRef.current = setTimeout;
    clearTimeoutRef.current = clearTimeout;
  }, [setTimeout, clearTimeout]);

  // React.useEffect(() => {
  //   timerRef.current = timer;
  //   setTimerRef.current = setTimer;
  // }, [timer, setTimerRef]);

  React.useEffect(() => {
    const uri =
      (location.protocol === 'https:' ? 'wss://' : 'ws://') +
      location.host +
      location.pathname +
      (location.pathname.endsWith('/') ? 'api' : '/api');

    const ws = new WebSocket(uri);
    setWebsocket(ws);

    ws.addEventListener('open', () =>
      updateStateRef.current({ connected: true }),
    );
    ws.addEventListener('close', () =>
      updateStateRef.current({ connected: false }),
    );
    ws.addEventListener('message', (event: MessageEvent) => {
      // console.log("ws response received : " + event.data);

      if (timerList.length > 0) {
        const timer = timerList.shift();
        // console.log("clear timer from receive: " + timer);
        clearTimeoutRef.current(timer);
      }
      // if (timerRef.current !== null) {
      //   console.log("clear timer from receive: " + timerRef.current);
      //   clearTimeoutRef.current(timerRef.current);
      // }

      // setTimerRef.current(null);

      const message = JSON.parse(event.data);
      if (message === 'Kicked') {
        ws.close();
      } else {
        updateStateRef.current({
          connected: true,
          ...websocketHandler(stateRef.current, message),
        });
      }
    });

    // return () => {
    //   if (timerRef.current !== null) {
    //     clearTimeoutRef.current(timerRef.current);
    //   }
    // };
  }, []);

  const send = (value: any) => {
    // if (timerRef.current !== null) {
    //   clearTimeoutRef.current(timerRef.current);
    //   console.log("clear timer from send: " + timerRef.current);
    // }
    // We expect a response back from the server within 5 seconds. Otherwise,
    // we should assume we have lost our websocket connection.
    const id = setTimeoutRef.current(() => {
      updateStateRef.current({ connected: false });
    }, 5000);
    // setTimerRef.current(
    //   id
    // );
    // console.log("set timer from send: " + id);
    const currentTimerList = timerList;
    currentTimerList.push(id);
    setTimerList(currentTimerList);
    // websocket?.send(JSON.stringify({...value, header_senderName: stateRef.current.name, header_timerId: id}));
    // console.log("send data: " + JSON.stringify({...value, shengji_header_senderName: stateRef.current.name, shengji_header_timerId: id}));
    websocket?.send(JSON.stringify(value));
    // console.log("send data: " + JSON.stringify(value));
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
