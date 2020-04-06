import * as React from 'react';
import {Settings} from './SettingsProvider';
import DivWithProps from './DivWithProps';

const Row = DivWithProps({style: {display: 'table-row'}});
const LabelCell = DivWithProps({
  style: {display: 'table-cell', paddingRight: '2em'},
});
const Cell = DivWithProps({style: {display: 'table-cell'}});

type Props = {
  settings: Settings;
  onChange: (settings: Settings) => void;
};
const SettingsPane = (props: Props) => {
  const {settings} = props;
  const handleChange = (partialSettings: Partial<Settings>) => () =>
    props.onChange({...props.settings, ...partialSettings});

  return (
    <div className="settings" style={{display: 'table'}}>
      <Row>
        <LabelCell>four-color mode</LabelCell>
        <Cell>
          <input
            name="four-color-mode"
            type="checkbox"
            checked={settings.fourColor}
            onChange={handleChange({fourColor: !settings.fourColor})}
          />
        </Cell>
      </Row>
      <Row>
        <LabelCell>show last trick</LabelCell>
        <Cell>
          <input
            name="show-last-trick"
            type="checkbox"
            checked={settings.showLastTrick}
            onChange={handleChange({showLastTrick: !settings.showLastTrick})}
          />
        </Cell>
      </Row>
      <Row>
        <LabelCell>beep on turn</LabelCell>
        <Cell>
          <input
            name="beep-on-turn"
            type="checkbox"
            checked={settings.beepOnTurn}
            onChange={handleChange({beepOnTurn: !settings.beepOnTurn})}
          />
        </Cell>
      </Row>
    </div>
  );
};

export default SettingsPane;
