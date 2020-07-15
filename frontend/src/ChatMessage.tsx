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
    default:
      return <span>{message.message}</span>;
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
