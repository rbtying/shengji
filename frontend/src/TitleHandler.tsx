import * as React from "react";
import { SettingsContext } from "./AppStateProvider";

const DEFAULT_TITLE = "Play 升级 / Tractor / 找朋友 / Finding Friends online!";

const TitleHandler = (props: { playerName?: string }): JSX.Element => {
  const settings = React.useContext(SettingsContext);
  React.useEffect(() => {
    if (
      props.playerName !== undefined &&
      props.playerName !== null &&
      settings.showPlayerName
    ) {
      document.title = `${props.playerName} | ${DEFAULT_TITLE}`;
    } else {
      document.title = DEFAULT_TITLE;
    }
  }, [props.playerName, settings.showPlayerName]);
  return <></>;
};

export default TitleHandler;
