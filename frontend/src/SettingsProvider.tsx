import * as React from 'react';

const loadSettings = (): Settings => ({
  fourColor: window.localStorage.getItem('four_color') == 'on' || false,
  beepOnTurn: window.localStorage.getItem('beep_on_turn') == 'on' || false,
  showLastTrick:
    window.localStorage.getItem('show_last_trick') == 'on' || false,
});

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
  defaultSettings?: Settings;
  children: (
    settings: Settings,
    handleChangeSettings: (settings: Settings) => void,
  ) => JSX.Element;
};

const SettingsProvider = ({
  defaultSettings = loadSettings(),
  children,
}: Props) => {
  const [settings, setSettings] = React.useState<Settings>(defaultSettings);
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
  return children(settings, handleChangeSettings);
};

export default SettingsProvider;
