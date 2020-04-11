import * as React from 'react';
import GameMode from './GameMode';
import SettingsButton from './SettingsButton';
import {IGameMode} from './types';

type Props = {
  gameMode: IGameMode;
  chatLink?: string | null;
};

const Header = (props: Props) => (
  <div>
    <h1>
      <GameMode gameMode={props.gameMode} />
      <SettingsButton />
    </h1>
    {props.chatLink ? (
      <p>
        Join the chat at{' '}
        <a href={props.chatLink} target="_blank">
          {props.chatLink}
        </a>
      </p>
    ) : null}
  </div>
);

export default Header;
