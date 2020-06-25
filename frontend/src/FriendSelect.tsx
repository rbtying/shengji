import * as React from "react";
import Select from "react-select";
import { ITrump } from "./types";
import ArrayUtils from "./util/array";
import preloadedCards from "./preloadedCards";
import InlineCard from "./InlineCard";
import { cardLookup } from "./util/cardHelpers";

interface FriendSelection {
  card: string;
  initial_skip: number;
}
interface IProps {
  friend: FriendSelection;
  trump: ITrump;
  num_decks: number;
  friend_selection_policy: string;
  onChange: (input: FriendSelection) => void;
}
interface Option {
  value: string;
  label: string;
}

const FriendSelect = (props: IProps): JSX.Element => {
  const handleChange = (transform: (e: Option) => Partial<FriendSelection>) => (
    value: Option
  ) => {
    props.onChange({
      card: props.friend.card,
      initial_skip: props.friend.initial_skip,
      ...transform(value),
    });
  };

  const handleCardChange = handleChange((select) => ({
    card: select.value,
  }));
  const handleOrdinalChange = handleChange((select) => ({
    initial_skip: parseInt(select.value, 10),
  }));

  const rank =
    props.trump.Standard !== undefined
      ? props.trump.Standard.number
      : props.trump.NoTrump.number;

  const cardOptions: Option[] = [];
  const currentValue: { [s: string]: any } = {};
  if (props.friend.card !== "") {
    const c = cardLookup[props.friend.card];
    currentValue.label = `${c.number}${c.typ}`;
    currentValue.value = c.value;
  }

  preloadedCards.forEach((c) => {
    if (
      c.number !== null &&
      c.number !== rank &&
      (props.trump.Standard == null || c.typ !== props.trump.Standard.suit)
    ) {
      // exclude highest card
      if (
        (props.friend_selection_policy === "HighestCardNotAllowed" &&
          ((rank !== "A" && c.number !== "A") ||
            (rank === "A" && c.number !== "K"))) ||
        props.friend_selection_policy === "Unrestricted"
      ) {
        cardOptions.push({
          label: `${c.number}${c.typ}`,
          value: c.value,
        });
      }
    }
  });

  return (
    <div className="friend-select">
      <div style={{ width: "100px", display: "inline-block" }}>
        <Select
          value={currentValue}
          onChange={handleCardChange}
          options={cardOptions}
          formatOptionLabel={({ value }) =>
            value !== undefined && value !== null && value !== "" ? (
              <InlineCard card={value} />
            ) : (
              value
            )
          }
        />
      </div>
      <div
        style={{ width: "100px", display: "inline-block", marginLeft: "10px" }}
      >
        <Select
          value={
            props.friend.initial_skip !== null
              ? {
                  value: `${props.friend.initial_skip}`,
                  label: `#${props.friend.initial_skip + 1}`,
                }
              : {}
          }
          onChange={handleOrdinalChange}
          options={ArrayUtils.range(props.num_decks, (idx) => {
            return { value: `${idx}`, label: `#${idx + 1}` };
          })}
        />
      </div>
    </div>
  );
};

export default FriendSelect;
