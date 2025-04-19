import * as React from "react";
import {
  Settings,
  ISuitOverrides,
  DEFAULT_POINT_CARD_ICON,
  DEFAULT_TRUMP_CARD_ICON,
} from "./state/Settings";
import { CompactPicker } from "react-color";
import styled from "styled-components";

import type { JSX } from "react";

const Picker = React.lazy(async () => await import("emoji-picker-react"));

const Row = styled.div`
  display: table-row;
  line-height: 23px;
`;
const LabelCell = styled.div`
  display: table-cell;
  padding-right: 2em;
`;
const Cell = styled.div`
  display: table-cell;
`;

interface IProps {
  settings: Settings;
  onChangeSettings: (settings: Settings) => void;
}

const SettingsPane = (props: IProps): JSX.Element => {
  const { settings } = props;
  const makeChangeHandler = (partialSettings: Partial<Settings>) => () => {
    const newSettings = { ...props.settings, ...partialSettings };
    props.onChangeSettings(newSettings);
  };

  const [link, setLink] = React.useState<string>("");

  const setChatLink = (event: React.SyntheticEvent): void => {
    event.preventDefault();
    if (link.length > 0) {
      (window as any).send({ Action: { SetChatLink: link } });
    } else {
      (window as any).send({ Action: { SetChatLink: null } });
    }
    setLink("");
  };

  const editor = (
    <div style={{ marginBottom: "15px" }}>
      <input
        type="text"
        style={{ width: "150px" }}
        value={link}
        onChange={(evt) => {
          evt.preventDefault();
          setLink(evt.target.value);
        }}
        placeholder="https://... link to voice chat"
      />
      <input type="button" onClick={setChatLink} value="set" />
    </div>
  );

  return (
    <div className="settings">
      <div style={{ display: "table" }}>
        <Row>
          <LabelCell>four-color mode</LabelCell>
          <Cell>
            <input
              name="four-color-mode"
              type="checkbox"
              checked={settings.fourColor}
              onChange={makeChangeHandler({ fourColor: !settings.fourColor })}
            />
          </Cell>
        </Row>
        <Row>
          <LabelCell>dark mode</LabelCell>
          <Cell>
            <input
              name="dark-mode"
              type="checkbox"
              checked={settings.darkMode}
              onChange={makeChangeHandler({ darkMode: !settings.darkMode })}
            />
          </Cell>
        </Row>
        <Row>
          <LabelCell>use SVG cards</LabelCell>
          <Cell>
            <input
              name="svg-cards"
              type="checkbox"
              checked={settings.svgCards}
              onChange={makeChangeHandler({ svgCards: !settings.svgCards })}
            />
          </Cell>
        </Row>
        <Row>
          <LabelCell>always show card labels</LabelCell>
          <Cell>
            <input
              name="show-card-labels"
              type="checkbox"
              checked={settings.showCardLabels}
              onChange={makeChangeHandler({
                showCardLabels: !settings.showCardLabels,
              })}
            />
          </Cell>
        </Row>
        <Row>
          <LabelCell>icon on point cards</LabelCell>
          <Cell>
            <EmojiPicker
              value={settings.pointCardIcon}
              setEmoji={(emoji) => {
                makeChangeHandler({
                  pointCardIcon: emoji,
                })();
              }}
              setDefault={makeChangeHandler({
                pointCardIcon: DEFAULT_POINT_CARD_ICON,
              })}
            />
          </Cell>
        </Row>
        <Row>
          <LabelCell>icon on trump cards</LabelCell>
          <Cell>
            <EmojiPicker
              value={settings.trumpCardIcon}
              setEmoji={(emoji) => {
                makeChangeHandler({
                  trumpCardIcon: emoji,
                })();
              }}
              setDefault={makeChangeHandler({
                trumpCardIcon: DEFAULT_TRUMP_CARD_ICON,
              })}
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
              onChange={makeChangeHandler({
                showLastTrick: !settings.showLastTrick,
              })}
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
              onChange={makeChangeHandler({ beepOnTurn: !settings.beepOnTurn })}
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
              onChange={makeChangeHandler({
                reverseCardOrder: !settings.reverseCardOrder,
              })}
            />
          </Cell>
        </Row>
        <Row>
          <LabelCell>separate cards by effective suit (in hand)</LabelCell>
          <Cell>
            <input
              name="separate-cards-by-suit"
              type="checkbox"
              checked={settings.separateCardsBySuit}
              onChange={makeChangeHandler({
                separateCardsBySuit: !settings.separateCardsBySuit,
              })}
            />
          </Cell>
        </Row>
        <Row>
          <LabelCell>disable suit highlights</LabelCell>
          <Cell>
            <input
              name="disable-suit-highlights"
              type="checkbox"
              checked={settings.disableSuitHighlights}
              onChange={makeChangeHandler({
                disableSuitHighlights: !settings.disableSuitHighlights,
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
              onChange={makeChangeHandler({
                unsetAutoPlayWhenWinnerChanges:
                  !settings.unsetAutoPlayWhenWinnerChanges,
              })}
            />
          </Cell>
        </Row>
        <Row>
          <LabelCell>show tricks in player order</LabelCell>
          <Cell>
            <input
              name="show-trick-in-player-order"
              type="checkbox"
              checked={settings.showTrickInPlayerOrder}
              onChange={makeChangeHandler({
                showTrickInPlayerOrder: !settings.showTrickInPlayerOrder,
              })}
            />
          </Cell>
        </Row>
        <Row>
          <LabelCell>suit color overrides</LabelCell>
          <Cell>
            {settings.svgCards ? (
              "disabled with SVG cards"
            ) : (
              <SuitOverrides
                suitColors={settings.suitColorOverrides}
                setSuitColors={(newOverrides: ISuitOverrides) =>
                  props.onChangeSettings({
                    ...props.settings,
                    suitColorOverrides: newOverrides,
                  })
                }
              />
            )}
          </Cell>
        </Row>
        <Row>
          <LabelCell>play sound when drawing card</LabelCell>
          <Cell>
            <input
              name="play-sound-when-drawing-card"
              type="checkbox"
              checked={settings.playDrawCardSound}
              onChange={makeChangeHandler({
                playDrawCardSound: !settings.playDrawCardSound,
              })}
            />
          </Cell>
        </Row>
        <Row>
          <LabelCell>show debugging information</LabelCell>
          <Cell>
            <input
              name="show-debug-info"
              type="checkbox"
              checked={settings.showDebugInfo}
              onChange={makeChangeHandler({
                showDebugInfo: !settings.showDebugInfo,
              })}
            />
          </Cell>
        </Row>
        <Row>
          <LabelCell>show player name in title bar</LabelCell>
          <Cell>
            <input
              name="show-player-name"
              type="checkbox"
              checked={settings.showPlayerName}
              onChange={makeChangeHandler({
                showPlayerName: !settings.showPlayerName,
              })}
            />
          </Cell>
        </Row>
        <Row>
          <LabelCell>hide chat box</LabelCell>
          <Cell>
            <input
              name="hide-chat-box"
              type="checkbox"
              checked={settings.hideChatBox}
              onChange={makeChangeHandler({
                hideChatBox: !settings.hideChatBox,
              })}
            />
          </Cell>
        </Row>
        <Row>
          <LabelCell>
            show points bar above the game (rather than below)
          </LabelCell>
          <Cell>
            <input
              name="show-points-above-game"
              type="checkbox"
              checked={settings.showPointsAboveGame}
              onChange={makeChangeHandler({
                showPointsAboveGame: !settings.showPointsAboveGame,
              })}
            />
          </Cell>
        </Row>
        <Row>
          <LabelCell>autodraw speed</LabelCell>
          <Cell>
            <select
              value={
                settings.autodrawSpeedMs !== null
                  ? settings.autodrawSpeedMs
                  : ""
              }
              onChange={(e) =>
                makeChangeHandler({
                  autodrawSpeedMs: parseInt(e.target.value),
                })()
              }
            >
              <option value="250">default</option>
              <option value="500">slow</option>
              <option value="10">fast</option>
            </select>
          </Cell>
        </Row>
      </div>
      <hr />
      <div style={{ display: "table" }}>
        <Row>
          <LabelCell>chat link</LabelCell>
          <Cell>{editor}</Cell>
        </Row>
      </div>
    </div>
  );
};

const SuitOverrides = (props: {
  suitColors: ISuitOverrides;
  setSuitColors: (overrides: ISuitOverrides) => void;
}): JSX.Element => {
  const suits: Array<keyof ISuitOverrides> = ["♢", "♡", "♤", "♧", "🃟", "🃏"];
  const labels = ["♦", "♥", "♠", "♣", "LJ", "HJ"];
  return (
    <>
      {suits.map((suit, idx) => (
        <SuitColorPicker
          key={suit}
          suit={suit}
          label={labels[idx]}
          suitColor={props.suitColors[suit]}
          setSuitColor={(color: string) => {
            const n = { ...props.suitColors };
            n[suit] = color;
            props.setSuitColors(n);
          }}
        />
      ))}
      <button
        className="normal"
        onClick={(evt) => {
          evt.preventDefault();
          props.setSuitColors({});
        }}
      >
        reset
      </button>
    </>
  );
};

const SuitColorPicker = (props: {
  suit: string;
  label: string;
  suitColor?: string;
  setSuitColor: (color: string) => void;
}): JSX.Element => {
  const [showPicker, setShowPicker] = React.useState<boolean>(false);
  return (
    <>
      <span
        className={props.suit}
        style={{ color: props.suitColor, cursor: "pointer" }}
        onClick={() => setShowPicker(true)}
      >
        {props.label}
      </span>
      {showPicker ? (
        <div style={{ position: "absolute" }}>
          <div
            style={{ position: "fixed", top: 0, left: 0, right: 0, bottom: 0 }}
            onClick={() => setShowPicker(false)}
          />
          <CompactPicker
            color={props.suitColor}
            onChangeComplete={(c: any) => props.setSuitColor(c.hex)}
          />
        </div>
      ) : null}
    </>
  );
};

const EmojiPicker = (props: {
  value: string;
  setEmoji: (emoji: string) => void;
  setDefault: () => void;
}): JSX.Element => {
  const [showPicker, setShowPicker] = React.useState<boolean>(false);
  return (
    <>
      <span>{props.value}</span>
      {!showPicker && (
        <button className="normal" onClick={() => setShowPicker(true)}>
          pick
        </button>
      )}
      {showPicker && (
        <button className="normal" onClick={() => setShowPicker(false)}>
          hide
        </button>
      )}
      <button className="normal" onClick={props.setDefault}>
        reset
      </button>
      {props.value !== "" && (
        <button className="normal" onClick={() => props.setEmoji("")}>
          no icon
        </button>
      )}
      {showPicker && (
        <React.Suspense fallback={"..."}>
          <Picker
            onEmojiClick={(emoji) => {
              props.setEmoji(emoji.emoji);
            }}
          />
        </React.Suspense>
      )}
    </>
  );
};

export default SettingsPane;
