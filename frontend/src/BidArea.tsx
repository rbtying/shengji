import * as React from "react";
import Cards from "./Cards";
import {
  Bid,
  Player,
  Hands,
  Trump,
  BidPolicy,
  BidReinforcementPolicy,
  JokerBidPolicy,
} from "./gen-types";
import { WebsocketContext } from "./WebsocketProvider";
import LabeledPlay from "./LabeledPlay";
import WasmContext from "./WasmContext";

interface IBidAreaProps {
  bids: Bid[];
  autobid: Bid | null;
  trump?: Trump;
  epoch: number;
  name: string;
  landlord: number | null;
  players: Player[];
  header?: JSX.Element | JSX.Element[];
  prefixButtons?: JSX.Element | JSX.Element[];
  suffixButtons?: JSX.Element | JSX.Element[];
  bidTakeBacksEnabled: boolean;
  bidPolicy: BidPolicy;
  bidReinforcementPolicy: BidReinforcementPolicy;
  jokerBidPolicy: JokerBidPolicy;
  hands: Hands;
  numDecks: number;
}

const BidArea = (props: IBidAreaProps): JSX.Element => {
  const { send } = React.useContext(WebsocketContext);
  const { findValidBids } = React.useContext(WasmContext);
  const trump = props.trump == null ? { NoTrump: {} } : props.trump;

  const takeBackBid = (evt: React.SyntheticEvent): void => {
    evt.preventDefault();
    send({ Action: "TakeBackBid" });
  };

  const players: { [playerId: number]: Player } = {};
  let playerId = -1;
  props.players.forEach((p: Player): void => {
    players[p.id] = p;
    if (p.name === props.name) {
      playerId = p.id;
    }
  });

  if (playerId === null || playerId < 0) {
    // Spectator mode
    return (
      <div>
        {props.header}
        {props.autobid !== null ? (
          <LabeledPlay
            label={`${players[props.autobid.id].name} (from bottom)`}
            trump={trump}
            cards={[props.autobid.card]}
          />
        ) : null}
        {props.bids.map((bid, idx) => {
          const name = players[bid.id].name;
          return (
            <LabeledPlay
              label={name}
              key={idx}
              trump={trump}
              cards={Array(bid.count).fill(bid.card)}
            />
          );
        })}
        {props.bids.length === 0 && props.autobid === null ? (
          <LabeledPlay trump={trump} label={"No bids yet..."} cards={["🂠"]} />
        ) : null}
      </div>
    );
  } else {
    const validBids = findValidBids({
      id: playerId,
      bids: props.bids,
      hands: props.hands,
      players: props.players,
      landlord: props.landlord,
      epoch: props.epoch,
      bid_policy: props.bidPolicy,
      bid_reinforcement_policy: props.bidReinforcementPolicy,
      joker_bid_policy: props.jokerBidPolicy,
      num_decks: props.numDecks,
    });
    const levelId =
      props.landlord !== null && props.landlord !== undefined
        ? props.landlord
        : playerId;

    const trump: any =
      props.trump !== null && props.trump !== undefined
        ? props.trump
        : {
            NoTrump: {
              number:
                players[levelId].level !== "NT" ? players[levelId].level : null,
            },
          };

    validBids.sort((a, b) => {
      if (a.card < b.card) {
        return -1;
      } else if (a.card > b.card) {
        return 1;
      } else if (a.count < b.count) {
        return -1;
      } else if (a.count > b.count) {
        return 1;
      } else {
        return 0;
      }
    });

    return (
      <div>
        <div>
          {props.header}
          {props.autobid !== null ? (
            <LabeledPlay
              label={`${players[props.autobid.id].name} (from bottom)`}
              cards={[props.autobid.card]}
              trump={trump}
            />
          ) : null}
          {props.bids.map((bid, idx) => {
            const name = players[bid.id].name;
            return (
              <LabeledPlay
                label={name}
                key={idx}
                trump={trump}
                cards={Array(bid.count).fill(bid.card)}
              />
            );
          })}
          {props.trump !== undefined &&
          "NoTrump" in props.trump &&
          props.trump?.NoTrump?.number === null ? (
            <>No bidding in no trump!</>
          ) : props.bids.length === 0 && props.autobid === null ? (
            <LabeledPlay trump={trump} label={"No bids yet..."} cards={["🂠"]} />
          ) : null}
        </div>
        {props.prefixButtons}
        {props.bidTakeBacksEnabled ? (
          <button
            onClick={takeBackBid}
            disabled={
              props.bids.length === 0 ||
              props.bids[props.bids.length - 1].id !== playerId ||
              props.bids[props.bids.length - 1].epoch !== props.epoch
            }
          >
            Take back bid
          </button>
        ) : null}
        {props.suffixButtons}
        {validBids.length > 0 ? (
          <p>Click a bid option to bid</p>
        ) : (
          <p>No available bids!</p>
        )}
        {validBids.map((bid, idx) => {
          return (
            <LabeledPlay
              trump={trump}
              cards={Array(bid.count).fill(bid.card)}
              key={idx}
              label={`Bid option ${idx + 1}`}
              onClick={() => {
                send({ Action: { Bid: [bid.card, bid.count] } });
              }}
            />
          );
        })}
        <Cards hands={props.hands} playerId={playerId} trump={trump} />
      </div>
    );
  }
};

export default BidArea;
