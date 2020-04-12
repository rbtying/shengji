import * as React from 'react';
import {TimerConsumer} from './TimerProvider';

type InnerProps = {
  timeout: number;
  callback: () => void;
  setTimeout: (fn: () => void, timeout: number) => number;
  clearTimeout: (id: number) => void;
};

const _Timeout = (props: InnerProps): null => {
  React.useEffect(() => {
    const timeout = props.setTimeout(props.callback, props.timeout);
    return () => props.clearTimeout(timeout);
  });
  return null;
};

type Props = {
  timeout: number;
  callback: () => void;
};

const Timeout = (props: Props) => (
  <TimerConsumer>
    {({setTimeout, clearTimeout}) => (
      <_Timeout
        timeout={props.timeout}
        callback={props.callback}
        setTimeout={setTimeout}
        clearTimeout={clearTimeout}
      />
    )}
  </TimerConsumer>
);

export default Timeout;
