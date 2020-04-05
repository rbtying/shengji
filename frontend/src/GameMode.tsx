import * as React from 'react';
import {IGameMode} from './types';

type Props = {
  gameMode: IGameMode;
};
const GameMode = (props: Props) => {
  const rules = (
    <a href="rules" target="_blank">
      rules
    </a>
  );
  if (props.gameMode == 'Tractor') {
    return <h1>升级 / Tractor ({rules})</h1>;
  } else {
    return <h1>找朋友 / Finding Friends ({rules})</h1>;
  }
};

export default GameMode;
