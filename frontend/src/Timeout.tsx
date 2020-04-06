import * as React from 'react';

type Props = {
  timeout: number;
  callback: () => void;
};

const Timeout = (props: Props): null => {
  React.useEffect(() => {
    const timeout = setTimeout(props.callback, props.timeout);
    return () => clearTimeout(timeout);
  });
  return null;
};

export default Timeout;
