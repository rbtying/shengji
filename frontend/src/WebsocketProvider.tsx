import * as React from "react";
import { AppStateContext } from "./AppStateProvider";
import websocketHandler from "./websocketHandler";
import { TimerContext } from "./TimerProvider";
import memoize from "./memoize";
import WasmContext from "./WasmContext";

interface Context {
  send: (value: any) => void;
}

export const WebsocketContext = React.createContext<Context>({
  send: () => {},
});

interface IProps {
  children: JSX.Element[] | JSX.Element;
}

interface IBlobToArrayBufferQueue {
  enqueue: (blob: Blob, handler: (arr: ArrayBuffer) => void) => void;
}

const getFileReader: () => IBlobToArrayBufferQueue = memoize(() => {
  const queue: Array<{ blob: Blob; handler: (arr: ArrayBuffer) => void }> = [];
  const fr = new FileReader();
  fr.onload = () => {
    const next = queue.shift();
    next.handler(fr.result as ArrayBuffer);
    if (queue.length > 0) {
      fr.readAsArrayBuffer(queue[0].blob);
    }
  };
  return {
    enqueue: (blob: Blob, handler: (arr: ArrayBuffer) => void) => {
      queue.push({ blob, handler });
      if (
        queue.length > 0 &&
        (fr.readyState === FileReader.EMPTY ||
          fr.readyState === FileReader.DONE)
      ) {
        fr.readAsArrayBuffer(queue[0].blob);
      }
    },
  };
});

const getBlobArrayBuffer: () => IBlobToArrayBufferQueue = memoize(() => {
  const queue: Array<{ blob: Blob; handler: (arr: ArrayBuffer) => void }> = [];
  const inflight: number[] = [];
  const onload = (arr: ArrayBuffer): void => {
    const next = queue.shift();
    inflight.shift();
    next.handler(arr);
    if (queue.length > 0) {
      inflight.push(0);
      queue[0].blob.arrayBuffer().then(onload, (err) => console.log(err));
    }
  };
  return {
    enqueue: (blob: Blob, handler: (arr: ArrayBuffer) => void) => {
      queue.push({ blob, handler });
      if (inflight.length === 0 && queue.length > 0) {
        inflight.push(0);
        blob.arrayBuffer().then(onload, (err) => console.log(err));
      }
    },
  };
});

const WebsocketProvider: React.FunctionComponent<IProps> = (props: IProps) => {
  const { state, updateState } = React.useContext(AppStateContext);
  const { decodeWireFormat } = React.useContext(WasmContext);
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
    const runtimeWebsocketHost = (window as any)._WEBSOCKET_HOST;
    const uri =
      runtimeWebsocketHost !== undefined && runtimeWebsocketHost !== null
        ? runtimeWebsocketHost
        : (location.protocol === "https:" ? "wss://" : "ws://") +
          location.host +
          location.pathname +
          (location.pathname.endsWith("/") ? "api" : "/api");

    const ws = new WebSocket(uri);
    setWebsocket(ws);

    ws.addEventListener("open", () =>
      updateStateRef.current({ connected: true, everConnected: true })
    );
    ws.addEventListener("close", () =>
      updateStateRef.current({ connected: false })
    );
    ws.addEventListener("message", (event: MessageEvent) => {
      if (timerRef.current !== null) {
        clearTimeoutRef.current(timerRef.current);
      }
      setTimerRef.current(null);

      const f = (buf: ArrayBuffer): void => {
        const message = decodeWireFormat(new Uint8Array(buf));
        if (message === "Kicked") {
          ws.close();
        } else {
          updateStateRef.current({
            connected: true,
            everConnected: true,
            ...websocketHandler(stateRef.current, message, (msg) => {
              ws.send(JSON.stringify(msg));
            }),
          });
        }
      };

      if (event.data.arrayBuffer !== undefined) {
        const b2a = getBlobArrayBuffer();
        b2a.enqueue(event.data, f);
      } else {
        const frs = getFileReader();
        frs.enqueue(event.data, f);
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
