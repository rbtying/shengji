import * as React from "react";

import LabeledPlay from "./LabeledPlay";
import TrickDisplay from "./Trick";
import { Player, Trick, Trump } from "./gen-types";

import type { JSX } from "react";

interface CommonProps {
  players: Player[];
  landlord: number;
  landlordSuffix: string;
  landlordsTeam: number[];
  name: string;
  trump: Trump;
}

interface IProps extends CommonProps {
  currentTrick: JSX.Element;
  lastTrick?: Trick | null;
  ownHistory: string[][];
  ownHistoryAvailable: boolean;
}

export const PreviousTrickPanel = (
  props: CommonProps & { trick: Trick },
): JSX.Element => (
  <section
    id="previous-trick-panel"
    className="play-history-panel"
    aria-label="Previous trick"
  >
    <h3>Previous trick / 上一轮出牌</h3>
    <TrickDisplay
      trick={props.trick}
      players={props.players}
      landlord={props.landlord}
      landlord_suffix={props.landlordSuffix}
      landlords_team={props.landlordsTeam}
      name={props.name}
      showTrickInPlayerOrder={true}
    />
  </section>
);

export const OwnPlayHistoryPanel = (props: {
  history: string[][];
  trump: Trump;
}): JSX.Element => (
  <section
    id="own-play-history-panel"
    className="play-history-panel"
    aria-label="My play history"
  >
    <h3>My play history / 我的出牌历史</h3>
    {props.history.length === 0 ? (
      <p>No completed plays yet / 暂无已完成的出牌</p>
    ) : (
      <div className="own-play-history-list">
        {props.history
          .map((cards, index) => ({ cards, trickNumber: index + 1 }))
          .reverse()
          .map(({ cards, trickNumber }) => (
            <LabeledPlay
              key={trickNumber}
              trump={props.trump}
              cards={cards}
              label={`Trick ${trickNumber} / 第${trickNumber}墩`}
            />
          ))}
      </div>
    )}
  </section>
);

const PlayHistory = (props: IProps): JSX.Element => {
  const [showPreviousTrick, setShowPreviousTrick] = React.useState(false);
  const [showOwnHistory, setShowOwnHistory] = React.useState(false);
  const hasPreviousTrick =
    props.lastTrick !== undefined && props.lastTrick !== null;

  return (
    <div className="trick-history-layout">
      <div className="trick-history-main">
        <div className="current-trick-display">{props.currentTrick}</div>
        <div className="play-history-buttons" aria-label="Play history">
          <button
            type="button"
            className="big"
            disabled={!hasPreviousTrick}
            aria-expanded={showPreviousTrick}
            aria-controls="previous-trick-panel"
            onClick={() => setShowPreviousTrick((show) => !show)}
          >
            {hasPreviousTrick
              ? showPreviousTrick
                ? "Hide previous trick ▲ / 收起上一轮"
                : "Previous trick ▼ / 查看上一轮"
              : "Previous trick unavailable / 上一轮暂无"}
          </button>
          <button
            type="button"
            className="big"
            disabled={!props.ownHistoryAvailable}
            aria-expanded={showOwnHistory}
            aria-controls="own-play-history-panel"
            onClick={() => setShowOwnHistory((show) => !show)}
          >
            {props.ownHistoryAvailable
              ? showOwnHistory
                ? "Hide my history ▲ / 收起我的历史"
                : `My play history (${props.ownHistory.length}) ▼ / 我的出牌历史（${props.ownHistory.length}墩）`
              : "My play history unavailable / 观察者无个人记录"}
          </button>
        </div>
      </div>
      {showPreviousTrick && props.lastTrick ? (
        <PreviousTrickPanel
          trick={props.lastTrick}
          players={props.players}
          landlord={props.landlord}
          landlordSuffix={props.landlordSuffix}
          landlordsTeam={props.landlordsTeam}
          name={props.name}
          trump={props.trump}
        />
      ) : null}
      {showOwnHistory && props.ownHistoryAvailable ? (
        <OwnPlayHistoryPanel history={props.ownHistory} trump={props.trump} />
      ) : null}
    </div>
  );
};

export default PlayHistory;
