import * as React from "react";
import { IBroadcastMessage } from "./types";
import InlineCard from "./InlineCard";
import classNames from "classnames";
import ArrayUtil from "./util/array";

export interface IMessage {
  from: string;
  message: string;
  from_game?: boolean;
  data?: IBroadcastMessage;
}

const renderMessage = (message: IMessage): JSX.Element => {
  const variant = message.data?.variant;
  switch (variant?.type) {
    case "MadeBid":
      return (
        <span>
          {message.data.actor_name} bid{" "}
          {ArrayUtil.range(variant.count, (i) => (
            <InlineCard card={variant.card} key={i} />
          ))}
        </span>
      );
    case "PlayedCards":
      return (
        <span>
          {message.data.actor_name} played{" "}
          {variant.cards.map((card, i) => (
            <InlineCard card={card} key={i} />
          ))}
        </span>
      );
    case "EndOfGameKittyReveal":
      return (
        <span>
          {variant.cards.map((card, i) => (
            <InlineCard card={card} key={i} />
          ))}{" "}
          in kitty
        </span>
      );
    case "GameScoringParametersChanged":
      return renderScoringMessage(message);
    default:
      return <span>{message.message}</span>;
  }
};

const renderScoringMessage = (message: IMessage): JSX.Element => {
  const changes = [];
  const variant = message.data?.variant;
  if (variant?.type === "GameScoringParametersChanged") {
    if (
      variant.old_parameters.step_size_per_deck !==
      variant.parameters.step_size_per_deck
    ) {
      changes.push(
        <span key={changes.length}>
          step size: {variant.parameters.step_size_per_deck}分 per deck
        </span>
      );
    }
    if (
      variant.old_parameters.deadzone_size !== variant.parameters.deadzone_size
    ) {
      changes.push(
        <span key={changes.length}>
          non-leveling steps: {variant.parameters.deadzone_size}{" "}
        </span>
      );
    }
    if (
      variant.old_parameters.num_steps_to_non_landlord_turnover !==
      variant.parameters.num_steps_to_non_landlord_turnover
    ) {
      changes.push(
        <span key={changes.length}>
          steps to turnover:{" "}
          {variant.parameters.num_steps_to_non_landlord_turnover}{" "}
        </span>
      );
    }
    for (const k in variant.parameters.step_adjustments) {
      const adj = variant.parameters.step_adjustments[k];
      if (adj !== variant.old_parameters.step_adjustments[k]) {
        changes.push(
          <span key={changes.length}>
            step size adjustment for {k} decks set to {adj}{" "}
          </span>
        );
      }
    }
    for (const k in variant.old_parameters.step_adjustments) {
      const adj = variant.parameters.step_adjustments[k];
      if (adj === undefined || adj === null || adj === 0) {
        changes.push(
          <span key={changes.length}>adjustment for {k} decks removed </span>
        );
      }
    }
    if (
      variant.old_parameters.bonus_level_policy !==
      variant.parameters.bonus_level_policy
    ) {
      if (
        variant.parameters.bonus_level_policy ===
        "BonusLevelForSmallerLandlordTeam"
      ) {
        changes.push(
          <span key={changes.length}>small-team bonus enabled</span>
        );
      } else {
        changes.push(
          <span key={changes.length}>small-team bonus disabled</span>
        );
      }
    }
    return (
      <span>
        {message.data.actor_name} updated the scoring parameters: {changes}
      </span>
    );
  } else {
    return null;
  }
};

interface IProps {
  message: IMessage;
}
const ChatMessage = (props: IProps): JSX.Element => {
  const { message } = props;
  return (
    <>
      {message.data?.variant.type === "StartingGame" ? (
        <p
          className={classNames("message", {
            "game-message": message.from_game,
          })}
        >
          🚜 🚜 🚜 🚜 🚜 🚜 🚜 🚜 🚜 🚜 🚜 🚜
        </p>
      ) : null}
      <p
        className={classNames("message", { "game-message": message.from_game })}
      >
        <span>{message.from}: </span> {renderMessage(message)}
      </p>
    </>
  );
};

export default ChatMessage;
