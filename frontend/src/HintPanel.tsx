import * as React from "react";
import { AppStateContext } from "./AppStateProvider";
import { WebsocketContext } from "./WebsocketProvider";
import InlineCard from "./InlineCard";

import type { JSX } from "react";

interface IProps {
  /** Changes whenever the decision in front of the player changes; hides stale hints. */
  contextKey: string;
  /** Whether asking for a hint makes sense right now. */
  enabled: boolean;
  /** Offered as a button to select the suggested cards, when given. */
  onSelect?: (cards: string[]) => void;
}

const HintPanel = (props: IProps): JSX.Element => {
  const { send } = React.useContext(WebsocketContext);
  const { state, updateState } = React.useContext(AppStateContext);
  const [askedFor, setAskedFor] = React.useState<string | null>(null);

  const current = askedFor === props.contextKey;
  const ask = (): void => {
    setAskedFor(props.contextKey);
    updateState({ hint: null, hintPending: true });
    send("RequestHint");
  };

  const hint = current ? state.hint : null;
  const pending = current && state.hintPending;

  return (
    <div className="hint-panel">
      <button
        className="normal"
        onClick={ask}
        disabled={!props.enabled || pending}
        title="Suggest a move by simulating how the hand could play out"
      >
        {pending ? "Thinking…" : "Hint"}
      </button>
      {current && !pending && hint === null && (
        <span> No suggestion right now.</span>
      )}
      {hint !== null && (
        <div className="hint-body">
          <p>{hint.summary}</p>
          {hint.options.length > 1 && (
            <table className="hint-options">
              <thead>
                <tr>
                  <th>Play</th>
                  <th>Your side wins</th>
                  <th>Defenders&apos; points (avg)</th>
                </tr>
              </thead>
              <tbody>
                {hint.options.map((o, i) => (
                  <tr key={i}>
                    <td>
                      {o.cards.map((c, j) => (
                        <InlineCard key={j} card={c} />
                      ))}
                    </td>
                    <td>{Math.round(o.win_rate * 100)}%</td>
                    <td>{Math.round(o.defender_points)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
          {props.onSelect !== undefined && (
            <button
              className="normal"
              onClick={() => props.onSelect!(hint.cards)}
            >
              Select suggested cards
            </button>
          )}
        </div>
      )}
    </div>
  );
};

export default HintPanel;
