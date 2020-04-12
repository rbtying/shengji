import * as React from 'react';
import {IGameMode} from './types';
import classNames from 'classnames';

type MessageVariant =
  | 'ResettingGame'
  | 'StartingGame'
  | {TrickWon: {winner: number; points: number}}
  | {RankAdvanced: {player: number; new_rank: number}}
  | {NewLandlordForNextGame: {landlord: number}}
  | {PointsInKitty: {points: number; multiplier: number}}
  | {JoinedGame: {player: number}}
  | {JoinedTeam: {player: number}}
  | {LeftGame: {name: string}}
  | {KittySizeSet: {size: number | null}}
  | {NumDecksSet: {num_decks: number | null}}
  | {NumFriendsSet: {num_friends: number | null}}
  | {GameModeSet: {game_mode: IGameMode}}
  | 'TookBackPlay'
  | {SetDefendingPointVisibility: {visible: boolean}}
  | {SetLandlord: {landlord: number | null}}
  | {SetRank: {rank: string}}
  | {MadeBid: {card: string; count: number}};

type BroadcastMessage = {
  actor: number;
  variant: MessageVariant;
};

export type Message = {
  from: string;
  message: string;
  from_game?: boolean;
  data?: BroadcastMessage;
};

type Props = {
  message: Message;
};
const ChatMessage = (props: Props) => {
  const {message} = props;
  return (
    <p className={classNames('message', {'game-message': message.from_game})}>
      {message.from}: {message.message}
    </p>
  );
};

export default ChatMessage;
