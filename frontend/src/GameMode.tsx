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
  if (props.gameMode === 'Tractor') {
    return <span>升级 / Tractor ({rules})</span>;
  } else {
    return <span>找朋友 / Finding Friends ({rules})</span>;
  }
};

export default GameMode;
