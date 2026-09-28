import * as React from "react";
import classNames from "classnames";
import { face } from "./cards";
import SvgBigJoker from "../generated/playing-cards/1J";
import SvgSmallJoker from "../generated/playing-cards/2J";

import type { JSX } from "react";

interface IProps {
  card: string;
  small?: boolean;
  pressed?: boolean;
  suggested?: boolean;
  onClick?: () => void;
  style?: React.CSSProperties;
}

/** A playing card drawn with type, sized for phones. Buttons when tappable. */
const MiniCard = (props: IProps): JSX.Element => {
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
  const className = classNames("tta-card", {
    red: f.red,
    small: props.small,
    joker: f.rank === "★",
    points: f.points > 0,
    suggested: props.suggested,
  });
  // Jokers use the deck's jester artwork: colored for the big joker, black for the small.
  const Joker =
    props.card === "🃏"
      ? SvgBigJoker
      : props.card === "🃟"
        ? SvgSmallJoker
        : null;
  const content =
    Joker !== null ? (
      <Joker className="art" width="100%" height="100%" aria-hidden="true" />
    ) : (
      <>
        <span>{f.rank}</span>
        <span className="suit">{f.suit}</span>
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
