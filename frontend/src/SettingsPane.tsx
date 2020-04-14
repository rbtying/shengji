import * as React from 'react';
import {Settings} from './state/Settings';
import styled from 'styled-components';

const Row = styled.div`
  display: 'table-row';
  line-height: 23px;
`;
const LabelCell = styled.div`
  display: table-cell;
  padding-right: 2em;
`;
const Cell = styled.div`
  display: table-cell;
`;

type Props = {
  settings: Settings;
  onChangeSettings: (settings: Settings) => void;
};

const SettingsPane = (props: Props) => {
  const {settings} = props;
  const handleChange = (partialSettings: Partial<Settings>) => () =>
    props.onChangeSettings({...props.settings, ...partialSettings});

  const [link, setLink] = React.useState<string>('');

  const setChatLink = (event: React.SyntheticEvent) => {
    event.preventDefault();
    if (link.length > 0) {
      (window as any).send({Action: {SetChatLink: link}});
    } else {
      (window as any).send({Action: {SetChatLink: null}});
    }
    setLink('');
  };

  const editor = (
    <div style={{marginBottom: '15px'}}>
      <input
        type="text"
        style={{width: '150px'}}
        value={link}
        onChange={(evt) => {
          evt.preventDefault();
          setLink(evt.target.value);
        }}
        placeholder="https://... link to voice chat"
      />
      <input type="button" onClick={setChatLink} value="Set chat link" />
    </div>
  );

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
      <Row>
        <LabelCell>reverse card order (in hand)</LabelCell>
        <Cell>
          <input
            name="reverse-card-order"
            type="checkbox"
            checked={settings.reverseCardOrder}
            onChange={handleChange({
              reverseCardOrder: !settings.reverseCardOrder,
            })}
          />
        </Cell>
      </Row>
      <Row>
        <LabelCell>unset auto-play if winner changes</LabelCell>
        <Cell>
          <input
            name="unset-auto-play-when-winner-changes"
            type="checkbox"
            checked={settings.unsetAutoPlayWhenWinnerChanges}
            onChange={handleChange({
              unsetAutoPlayWhenWinnerChanges: !settings.unsetAutoPlayWhenWinnerChanges,
            })}
          />
        </Cell>
      </Row>
      <Row>
        <LabelCell>chat link</LabelCell>
        <Cell>{editor}</Cell>
      </Row>
    </div>
  );
};

export default SettingsPane;
