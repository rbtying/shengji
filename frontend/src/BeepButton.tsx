import * as React from 'react';
import {WebsocketContext} from './WebsocketProvider';

const BeepButton = () => {
  const {send} = React.useContext(WebsocketContext);

  return (
    <button
      onClick={() =>
        confirm('Do you really want to send a beep to the current player?') &&
        send('Beep')
      }
    >
      🛎️
    </button>
  );
};

export default BeepButton;
