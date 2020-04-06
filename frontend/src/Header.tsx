import * as React from 'react';
import GameMode from './GameMode';
import SettingsButton from './SettingsButton';
import {IGameMode} from './types';

type Props = {
  gameMode: IGameMode;
};

const Header = (props: Props) => (
  <div>
    <h1>
      <GameMode gameMode={props.gameMode} />
      <SettingsButton />
    </h1>
  </div>
);

export default Header;
