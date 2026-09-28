import * as React from "react";
import classNames from "classnames";
import { face } from "./cards";
import { useCardStyle } from "./cardStyle";
import SvgBlackJoker from "./BlackJoker";
import SvgRedJoker from "./RedJoker";

import type { JSX } from "react";

// The full deck of card faces loads as its own chunk.
const SvgCard = React.lazy(async () => await import("../SvgCard"));

interface IProps {
  card: string;
  small?: boolean;
  pressed?: boolean;
  suggested?: boolean;
  onClick?: () => void;
  style?: React.CSSProperties;
}

/**
 * A playing card sized for phones: real card faces by default, or large rank and suit in the
 * "simple" style. Buttons when tappable.
 */
const MiniCard = (props: IProps): JSX.Element => {
  const cardStyle = useCardStyle();
  if (props.card === "🂠") {
    return (
      <div
        className={classNames("tta-card", "back", { small: props.small })}
        style={props.style}
        aria-label="Hidden card"
      />
    );
  }
  const f = face(props.card);
  const isJoker = f.rank === "★";
  const faces = cardStyle === "faces";
  const className = classNames("tta-card", {
    face: faces,
    red: f.red,
    small: props.small,
    joker: isJoker,
    points: f.points > 0 && !faces,
    suggested: props.suggested,
  });
  // Jokers are hard to tell apart when small, so they always say which one they are.
  const jokerTag = isJoker ? (
    <span className="joker-tag" aria-hidden="true">
      {props.card === "🃏" ? "BIG" : "SMALL"}
    </span>
  ) : null;
  // The two jokers are the same jester: red for the big joker, black for the small.
  const jokerArt = isJoker
    ? React.createElement(props.card === "🃏" ? SvgRedJoker : SvgBlackJoker, {
        className: "art",
        width: "100%",
        height: "100%",
        "aria-hidden": true,
      })
    : null;
  const simple = isJoker ? (
    jokerArt
  ) : (
    <>
      <span>{f.rank}</span>
      <span className="suit">{f.suit}</span>
    </>
  );
  const content = (
    <>
      {faces && isJoker ? (
        <span className="face-art" aria-hidden="true">
          {jokerArt}
        </span>
      ) : faces ? (
        <React.Suspense fallback={simple}>
          <span className="face-art" aria-hidden="true">
            <SvgCard card={props.card} fourColor={false} />
          </span>
        </React.Suspense>
      ) : (
        simple
      )}
      {jokerTag}
    </>
  );
  if (props.onClick !== undefined) {
    return (
      <button
        type="button"
        className={className}
        style={props.style}
        aria-label={f.label}
        aria-pressed={props.pressed === true}
        onClick={props.onClick}
      >
        {content}
      </button>
    );
  }
  return (
    <div className={className} style={props.style} aria-label={f.label}>
      {content}
    </div>
  );
};

export default MiniCard;
