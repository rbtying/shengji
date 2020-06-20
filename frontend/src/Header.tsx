import * as React from "react";
import GameMode from "./GameMode";
import GameStatisticsButton from "./GameStatisticsButton";
import SettingsButton from "./SettingsButton";
import { IGameModeSettings } from "./types";

type Props = {
  gameMode: IGameModeSettings;
  chatLink?: string | null;
};

const Header = (props: Props) => (
  <div>
    <h1>
      <GameMode gameMode={props.gameMode} />
      &nbsp;
      <SettingsButton />
      <GameStatisticsButton />
    </h1>
    {props.chatLink ? (
      <p>
        Join the chat at{" "}
        <a href={props.chatLink} target="_blank">
          {props.chatLink}
        </a>
      </p>
    ) : null}
  </div>
);

export default Header;
