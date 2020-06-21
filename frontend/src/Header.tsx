import * as React from "react";
import GameMode from "./GameMode";
import GameStatisticsButton from "./GameStatisticsButton";
import SettingsButton from "./SettingsButton";
import { IGameModeSettings } from "./types";

interface IProps {
  gameMode: IGameModeSettings;
  chatLink?: string | null;
}

const Header = (props: IProps): JSX.Element => (
  <div>
    <h1>
      <GameMode gameMode={props.gameMode} />
      &nbsp;
      <SettingsButton />
      <GameStatisticsButton />
    </h1>
    {props.chatLink !== undefined && props.chatLink !== null ? (
      <p>
        Join the chat at{" "}
        <a href={props.chatLink} target="_blank" rel="noreferrer">
          {props.chatLink}
        </a>
      </p>
    ) : null}
  </div>
);

export default Header;
