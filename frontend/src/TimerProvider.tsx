// Provides a WebWorker-based timer implementation which doesn't get
// wakeup-limited by the browser when the tab is running in the background.
//
// Relies on timer-worker.js to service the underlying timing requests.

import * as React from "react";

interface Context {
  setTimeout: (fn: () => void, delay: number) => number;
  clearTimeout: (id: number) => void;
  setInterval: (fn: () => void, interval: number) => number;
  clearInterval: (id: number) => void;
}

export const TimerContext = React.createContext<Context>({
  setTimeout: (_fn, _delay) => 0,
  clearTimeout: (_id) => {},
  setInterval: (_fn, _interval) => 0,
  clearInterval: (_id) => {},
});

interface IProps {
  children: JSX.Element[] | JSX.Element;
}

const _TimerProvider: React.FunctionComponent<IProps> = (props: IProps) => {
  const [worker, setWorker] = React.useState<Worker | null>(null);
  const timeoutId = React.useRef(0);
  const callbacks = React.useRef<Map<number, () => void>>(new Map());

  React.useEffect(() => {
    const timerWorker = new Worker("timer-worker.js");

    timerWorker.addEventListener("message", (evt) => {
      const data = evt.data;
      const id = data.id as number;
      if (callbacks.current.has(id)) {
        callbacks.current.get(id)();
      }
      if (data.variant === "timeout") {
        callbacks.current.delete(id);
      }
    });
    setWorker(timerWorker);
    return () => {
      timerWorker.terminate();
    };
  }, []);

  const setTimeout = (fn: () => void, delay: number | undefined): number => {
    timeoutId.current += 1;
    delay = delay === undefined ? 0 : delay;
    const id = timeoutId.current;
    callbacks.current.set(id, fn);
    if (worker !== null) {
      worker.postMessage({ command: "setTimeout", id, timeout: delay });
    }
    return id;
  };

  const clearTimeout = (id: number): void => {
    worker.postMessage({ command: "clearTimeout", id });
    callbacks.current.delete(id);
  };

  const setInterval = (
    fn: () => void,
    interval: number | undefined
  ): number => {
    timeoutId.current += 1;
    interval = interval === undefined ? 0 : interval;
    const id = timeoutId.current;
    callbacks.current.set(id, fn);
    if (worker !== null) {
      worker.postMessage({ command: "setInterval", id, interval });
    }
    return id;
  };

  const clearInterval = (id: number): void => {
    if (worker !== null) {
      worker.postMessage({ command: "clearInterval", id });
    }
    callbacks.current.delete(id);
  };

  return (
    <TimerContext.Provider
      value={{ setTimeout, clearTimeout, setInterval, clearInterval }}
    >
      {props.children}
    </TimerContext.Provider>
  );
};

const TimerProvider: React.FunctionComponent<IProps> = (props: IProps) => (
  <_TimerProvider>{props.children}</_TimerProvider>
);

export default TimerProvider;
