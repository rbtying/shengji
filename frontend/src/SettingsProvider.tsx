import * as React from 'react';

export type Settings = {
  fourColor: boolean;
  showLastTrick: boolean;
  beepOnTurn: boolean;
};

type ISettingsContext = {
  settings: Settings;
  updateSettings: (settings: Settings) => void;
};

export const SettingsContext = React.createContext<ISettingsContext>({
  settings: {
    fourColor: false,
    showLastTrick: false,
    beepOnTurn: false,
  },
  updateSettings: () => {},
});

type Props = {
  defaultSettings: Settings;
  children: React.ReactNode;
};

const SettingsProvider = (props: Props) => {
  const [settings, setSettings] = React.useState<Settings>(
    props.defaultSettings,
  );
  const updateSettings = (newSettings: Settings) => {
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
  return (
    <SettingsContext.Provider value={{settings, updateSettings}}>
      {props.children}
    </SettingsContext.Provider>
  );
};

export default SettingsProvider;
