import * as React from "react";
import { IBroadcastMessage } from "./types";
import InlineCard from "./InlineCard";
import classNames from "classnames";
import ArrayUtil from "./util/array";
import { firework } from "./firework";

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

    case "GamesetWinnerAnnoucement":
      return <span className="red">{message.message}</span>;

    default:
      return <span>{message.message}</span>;
  }
};

interface IProps {
  message: IMessage;
}
const ChatMessage = (props: IProps): JSX.Element => {
  const [fireworkStarted, setFireworkStarted] = React.useState<boolean>(false);
  const { message } = props;

  if (
    message.data?.variant.type === "GamesetWinnerAnnoucement" &&
    !fireworkStarted
  ) {
    setFireworkStarted(true);
    firework(10);
  }
  return (
    <p className={classNames("message", { "game-message": message.from_game })}>
      {message.data?.variant.type === "StartingGame" ? (
        <span>
          🚜 🚜 🚜 🚜 🚜 🚜 🚜 🚜 🚜 🚜 🚜 🚜
          <br />
          {message.from}:{" "}
        </span>
      ) : (
        <span>{message.from}: </span>
      )}
      {renderMessage(message)}
    </p>
  );
};

export default ChatMessage;
