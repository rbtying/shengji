import * as React from 'react';

type Context = {
  setTimeout: (fn: () => void, delay: number) => number;
  clearTimeout: (id: number) => void;
  setInterval: (fn: () => void, interval: number) => number;
  clearInterval: (id: number) => void;
};

const TimerContext = React.createContext<Context>({
  setTimeout: (fn, delay) => 0,
  clearTimeout: (id) => {},
  setInterval: (fn, interval) => 0,
  clearInterval: (id) => {},
});

export const TimerConsumer = TimerContext.Consumer;

const _TimerProvider: React.FunctionComponent<{}> = (props) => {
  const [worker, setWorker] = React.useState<Worker | null>(null);
  const timeoutId = React.useRef(0);
  const callbacks = React.useRef<{[id: number]: () => void}>({});

  React.useEffect(() => {
    const timerWorker = new Worker('timer-worker.js');

    timerWorker.addEventListener('message', (evt) => {
      const data = evt.data;
      const id = data.id as number;
      if (callbacks.current[id]) {
        callbacks.current[id]();
      }
      if (data.variant === 'timeout') {
        delete callbacks.current[id];
      }
    });
    setWorker(timerWorker);
  }, []);

  const setTimeout = (fn: () => void, delay: number) => {
    timeoutId.current += 1;
    delay = delay || 0;
    const id = timeoutId.current;
    callbacks.current[id] = fn;
    worker.postMessage({command: 'setTimeout', id, timeout: delay});
    return id;
  };

  const clearTimeout = (id: number) => {
    worker.postMessage({command: 'clearTimeout', id});
    delete callbacks.current[id];
  };

  const setInterval = (fn: () => void, interval: number) => {
    timeoutId.current += 1;
    interval = interval || 0;
    const id = timeoutId.current;
    callbacks.current[id] = fn;
    worker.postMessage({command: 'setInterval', id, interval});
    return id;
  };

  const clearInterval = (id: number) => {
    worker.postMessage({command: 'clearInterval', id});
    delete callbacks.current[id];
  };

  return (
    <TimerContext.Provider
      value={{setTimeout, clearTimeout, setInterval, clearInterval}}
    >
      {props.children}
    </TimerContext.Provider>
  );
};

const TimerProvider: React.FunctionComponent<{}> = (props) => (
  <_TimerProvider>{props.children}</_TimerProvider>
);

export default TimerProvider;
