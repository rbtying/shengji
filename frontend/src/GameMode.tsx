import * as React from 'react';
import {IGameModeSettings, IGameMode} from './types';

type Props = {
  gameMode: IGameModeSettings | IGameMode;
};
const GameMode = (props: Props) => {
  const rules = (
    <a href="rules" target="_blank">
      rules
    </a>
  );
  if (props.gameMode === 'Tractor') {
    return (
      <span>
        升级 / <span className="red">Tractor</span> ({rules})
      </span>
    );
  } else {
    return (
      <span>
        找朋友 / <span className="red">Finding Friends</span> ({rules})
      </span>
    );
  }
};

export default GameMode;
