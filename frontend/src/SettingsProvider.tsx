import * as React from 'react';

export type Settings = {
  fourColor: boolean;
  showLastTrick: boolean;
  beepOnTurn: boolean;
};

export type SettingsProps = {
  settings: Settings;
  onChangeSettings: (settings: Settings) => void;
};

type Props = {
  defaultSettings: Settings;
  children: (
    settings: Settings,
    handleChangeSettings: (settings: Settings) => void,
  ) => JSX.Element;
};

const SettingsProvider = (props: Props) => {
  const [settings, setSettings] = React.useState<Settings>(
    props.defaultSettings,
  );
  const handleChangeSettings = (newSettings: Settings) => {
    window.localStorage.setItem(
      'four_color',
      newSettings.fourColor ? 'on' : 'off',
    );
    window.localStorage.setItem(
      'show_last_trick',
      newSettings.showLastTrick ? 'on' : 'off',
    );
    window.localStorage.setItem(
      'beep_on_turn',
      newSettings.beepOnTurn ? 'on' : 'off',
    );
    setSettings(newSettings);
  };
  return props.children(settings, handleChangeSettings);
};

export default SettingsProvider;
