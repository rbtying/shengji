import * as React from 'react';
import GameMode from './GameMode';
import SettingsButton from './SettingsButton';
import {SettingsProps} from './SettingsProvider';
import {IGameMode} from './types';

type Props = SettingsProps & {
  gameMode: IGameMode;
};

const Header = (props: Props) => (
  <div>
    <h1>
      <GameMode gameMode={props.gameMode} />
      <SettingsButton
        settings={props.settings}
        onChangeSettings={props.onChangeSettings}
      />
    </h1>
  </div>
);

export default Header;
