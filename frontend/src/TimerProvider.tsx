// Provides a WebWorker-based timer implementation which doesn't get
// wakeup-limited by the browser when the tab is running in the background.
//
// Relies on timer-worker.js to service the underlying timing requests.

import * as React from 'react';

type Context = {
  setTimeout: (fn: () => void, delay: number) => number;
  clearTimeout: (id: number) => void;
  setInterval: (fn: () => void, interval: number) => number;
  clearInterval: (id: number) => void;
};

export const TimerContext = React.createContext<Context>({
  setTimeout: (fn, delay) => 0,
  clearTimeout: (id) => {},
  setInterval: (fn, interval) => 0,
  clearInterval: (id) => {},
});

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
    return () => {
      timerWorker.terminate();
    };
  }, []);

  const setTimeout = (fn: () => void, delay: number) => {
    timeoutId.current += 1;
    delay = delay || 0;
    const id = timeoutId.current;
    callbacks.current[id] = fn;
    if (worker !== null) {
      worker.postMessage({command: 'setTimeout', id, timeout: delay});
    }
    // console.log("TimerProvider set timeout: " + id);
    // console.trace();
    return id;
  };

  const clearTimeout = (id: number) => {
    // console.log("TimerProvider clear timeout: " + id);
    worker.postMessage({command: 'clearTimeout', id});
    delete callbacks.current[id];
  };

  const setInterval = (fn: () => void, interval: number) => {
    timeoutId.current += 1;
    interval = interval || 0;
    const id = timeoutId.current;
    callbacks.current[id] = fn;
    if (worker !== null) {
      worker.postMessage({command: 'setInterval', id, interval});
    }
    return id;
  };

  const clearInterval = (id: number) => {
    if (worker !== null) {
      worker.postMessage({command: 'clearInterval', id});
    }
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
