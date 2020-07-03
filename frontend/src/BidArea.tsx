import * as React from "react";
import Cards from "./Cards";
import { IBid, IPlayer } from "./types";
import { WebsocketContext } from "./WebsocketProvider";
import LabeledPlay from "./LabeledPlay";

interface IBidAreaProps {
  bids: IBid[];
  autobid: IBid | null;
  cards: string[];
  epoch: number;
  name: string;
  landlord: number | null;
  players: IPlayer[];
  separateBidCards: boolean;
  header?: JSX.Element | JSX.Element[];
  prefixButtons?: JSX.Element | JSX.Element[];
  suffixButtons?: JSX.Element | JSX.Element[];
  bidTakeBacksEnabled: boolean;
}

const BidArea = (props: IBidAreaProps): JSX.Element => {
  const [selected, setSelected] = React.useState<string[]>([]);
  const { send } = React.useContext(WebsocketContext);

  const makeBid = (evt: React.SyntheticEvent): void => {
    evt.preventDefault();
    const counts: { [card: string]: number } = {};
    selected.forEach(
      (c) => (counts[c] = (counts[c] !== undefined ? counts[c] : 0) + 1)
    );
    if (Object.keys(counts).length !== 1) {
      return;
    }

    const players: { [playerId: number]: IPlayer } = {};
    props.players.forEach((p: IPlayer) => {
      players[p.id] = p;
    });

    for (const c in counts) {
      let alreadyBid = 0;
      props.bids.forEach((bid: IBid) => {
        if (players[bid.id].name === props.name && bid.card === c) {
          alreadyBid = alreadyBid < bid.count ? bid.count : alreadyBid;
        }
      });

      send({ Action: { Bid: [c, counts[c] + alreadyBid] } });
      setSelected([]);
    }
  };

  const takeBackBid = (evt: React.SyntheticEvent): void => {
    evt.preventDefault();
    send({ Action: "TakeBackBid" });
  };

  const players: { [playerId: number]: IPlayer } = {};
  let playerId = -1;
  props.players.forEach((p: IPlayer): void => {
    players[p.id] = p;
    if (p.name === props.name) {
      playerId = p.id;
    }
  });

  const myBids: { [card: string]: number } = {};
  props.bids.forEach((bid: IBid): void => {
    if (playerId === bid.id) {
      const existingBid = bid.card in myBids ? myBids[bid.card] : 0;
      myBids[bid.card] = existingBid < bid.count ? bid.count : existingBid;
    }
  });
  const cardsNotBid = [...props.cards];

  Object.keys(myBids).forEach((card) => {
    const count = card in myBids ? myBids[card] : 0;
    for (let i = 0; i < count; i = i + 1) {
      const cardIdx = cardsNotBid.indexOf(card);
      if (cardIdx >= 0) {
        cardsNotBid.splice(cardIdx, 1);
      }
    }
  });

  const landlord = props.landlord;
  const level =
    props.landlord == null ? players[playerId].level : players[landlord].level;

  return (
    <div>
      <div>
        {props.header}
        {props.autobid !== null ? (
          <LabeledPlay
            label={`${players[props.autobid.id].name} (from bottom)`}
            cards={[props.autobid.card]}
          />
        ) : null}
        {props.bids.map((bid, idx) => {
          const name = players[bid.id].name;
          return (
            <LabeledPlay
              label={name}
              key={idx}
              cards={Array(bid.count).fill(bid.card)}
            />
          );
        })}
      </div>
      {props.prefixButtons}
      <button onClick={makeBid} disabled={selected.length === 0}>
        Make bid
      </button>
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
      {props.landlord !== null ? (
        <p>
          Bid using {players[props.landlord].level}
          &apos;s in the same suit, or jokers
        </p>
      ) : players[playerId] !== undefined ? (
        <p>
          Bid using {players[playerId].level}&apos;s in the same suit, or jokers
        </p>
      ) : (
        <div />
      )}
      <Cards
        cardsInHand={cardsNotBid}
        selectedCards={selected}
        onSelect={setSelected}
        separateBidCards={props.separateBidCards}
        level={level}
      />
    </div>
  );
};

export default BidArea;
