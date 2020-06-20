import * as React from "react";
import { IGameMode } from "./types";
import InlineCard from "./InlineCard";
import classNames from "classnames";
import ArrayUtil from "./util/array";

type MessageVariant =
  | { type: "GameModeSet"; game_mode: IGameMode }
  | { type: "JoinedGame"; player: number }
  | { type: "JoinedTeam"; player: number }
  | { type: "KittySizeSet"; size: number | null }
  | { type: "LeftGame"; name: string }
  | { type: "MadeBid"; card: string; count: number }
  | { type: "NewLandlordForNextGame"; landlord: number }
  | { type: "NumDecksSet"; num_decks: number | null }
  | { type: "NumFriendsSet"; num_friends: number | null }
  | { type: "PlayedCards"; cards: string[] }
  | { type: "PointsInKitty"; points: number; multiplier: number }
  | { type: "RankAdvanced"; player: number; new_rank: number }
  | { type: "ResettingGame" }
  | { type: "SetDefendingPointVisibility"; visible: boolean }
  | { type: "SetLandlord"; landlord: number | null }
  | { type: "SetRank"; rank: string }
  | { type: "StartingGame" }
  | { type: "TookBackPlay" }
  | { type: "TrickWon"; winner: number; points: number };

type BroadcastMessage = {
  actor: number;
  actor_name: string;
  variant: MessageVariant;
};

export type Message = {
  from: string;
  message: string;
  from_game?: boolean;
  data?: BroadcastMessage;
};

const renderMessage = (message: Message) => {
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
      const cards = variant.cards.map((card, i) => (
        <InlineCard card={card} key={i} />
      ));
      return (
        <span>
          {message.data.actor_name} played {cards}
        </span>
      );
    default:
      return <span>{message.message}</span>;
  }
};

type Props = {
  message: Message;
};
const ChatMessage = (props: Props) => {
  const { message } = props;
  return (
    <p className={classNames("message", { "game-message": message.from_game })}>
      <span>{message.from}: </span>
      {renderMessage(message)}
    </p>
  );
};

export default ChatMessage;
