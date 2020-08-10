import * as React from "react";
import { IGameScoringParameters } from "./types";
import { WebsocketContext } from "./WebsocketProvider";
import { WasmContext, IScoreSegment } from "./WasmContext";

interface IProps {
  params: IGameScoringParameters;
  numDecks: number;
}

/*

  step_size_per_deck: number;
  num_steps_to_non_landlord_turnover: number;
  deadzone_size: number;
  truncate_zero_crossing_window: boolean;
  bonus_level_policy: "NoBonusLevel" | "BonusLevelForSmallerLandlordTeam";
*/

export const GameScoringSettings = (props: IProps): JSX.Element => {
  const { send } = React.useContext(WebsocketContext);
  const { explainScoring } = React.useContext(WasmContext);
  const [highlighted, setHighlighted] = React.useState<number | null>(null);

  const updateSettings = (updates: Partial<IGameScoringParameters>): void => {
    send({
      Action: {
        SetGameScoringParameters: { ...props.params, ...updates },
      },
    });
  };

  const bonusEnabled =
    props.params.bonus_level_policy === "BonusLevelForSmallerLandlordTeam";

  const scoreTransitions = explainScoring({
    params: props.params,
    smaller_landlord_team_size: false,
    num_decks: props.numDecks,
  });

  const bonusScoreTransitions = bonusEnabled
    ? explainScoring({
        params: props.params,
        smaller_landlord_team_size: true,
        num_decks: props.numDecks,
      })
    : scoreTransitions;

  const scoreSegments: Array<{
    span: number;
    segment: IScoreSegment;
    bonusSegment: IScoreSegment | null;
  }> = [];
  let maxPts = 0;
  let maxLandlordDelta = 0;
  let maxNonLandlordDelta = 0;
  for (let i = 1; i < scoreTransitions.length; i++) {
    const span = Math.max(
      scoreTransitions[i].point_threshold -
        scoreTransitions[i - 1].point_threshold,
      10
    );
    const segment = scoreTransitions[i - 1];
    maxLandlordDelta = Math.max(
      segment.results.landlord_delta,
      maxLandlordDelta
    );
    maxNonLandlordDelta = Math.max(
      segment.results.non_landlord_delta,
      maxNonLandlordDelta
    );
    scoreSegments.push({
      span,
      segment,
      bonusSegment: bonusScoreTransitions[i - 1].results.landlord_bonus
        ? bonusScoreTransitions[i - 1]
        : null,
    });
    maxPts += span;
  }
  const last = scoreTransitions.length - 1;
  scoreSegments.push({
    span: 5 * props.numDecks,
    segment: scoreTransitions[last],
    bonusSegment: bonusScoreTransitions[last].results.landlord_bonus
      ? bonusScoreTransitions[last]
      : null,
  });
  maxPts += 5 * props.numDecks;

  const text = (idx: number): JSX.Element => {
    let txt = "Attacking team wins, but doesn't level up.";
    const segment = scoreSegments[idx];
    if (segment.segment.results.landlord_won) {
      txt = `Defending team wins, and goes up ${segment.segment.results.landlord_delta} levels.`;
      if (segment.bonusSegment) {
        txt += ` If the team is unexpectedly small, they go up ${segment.bonusSegment.results.landlord_delta} levels.`;
      }
    } else if (segment.segment.results.non_landlord_delta > 0) {
      txt = `Attacking team wins, and goes up ${segment.segment.results.non_landlord_delta} levels.`;
    }
    return <>{txt}</>;
  };

  const totalPoints = props.numDecks * 100;
  const validStepSizes = [];
  for (
    let curStepSize = 0;
    curStepSize <= totalPoints / 3;
    curStepSize += 5 * props.numDecks
  ) {
    if (curStepSize == 0) {
      continue;
    }
    if (totalPoints % curStepSize == 0) {
      validStepSizes.push(`${curStepSize}`);
    }
  }
  const maxSteps = Math.floor(100 / props.params.step_size_per_deck);

  return (
    <>
      <div>
        <div style={{ width: "95%", padding: "5px 0 5px 0" }}>
          {scoreSegments.map((segment, idx) => {
            const frac = segment.span / maxPts;
            let bg = "rgb(255, 255, 0)";
            if (segment.segment.results.landlord_won) {
              const f =
                segment.segment.results.landlord_delta / maxLandlordDelta;
              bg = `rgba(0, 255, 0, ${f})`;
            } else if (segment.segment.results.non_landlord_delta > 0) {
              const f =
                segment.segment.results.non_landlord_delta /
                maxNonLandlordDelta;
              bg = `rgba(255, 0, 0, ${f})`;
            }
            return (
              <div
                key={idx}
                onMouseEnter={(evt) => {
                  setHighlighted(idx);
                }}
                onMouseLeave={(evt) => {
                  setHighlighted(null);
                }}
                style={{
                  width: `${100 * frac}%`,
                  background: bg,
                  padding: "5px 0 5px 0",
                  display: "inline-block",
                  cursor: "pointer",
                }}
              >
                {segment.segment.point_threshold}
              </div>
            );
          })}
          {highlighted !== null ? (
            <p> {text(highlighted)}</p>
          ) : (
            <p>Hover over the attacking team scores for details</p>
          )}
        </div>
        <div>
          <label>Step size: </label>
          <select
            value={`${props.params.step_size_per_deck * props.numDecks}`}
            onChange={(evt) => {
              evt.preventDefault();
              const perDeck = parseInt(evt.target.value, 10) / props.numDecks;

              updateSettings({
                step_size_per_deck: perDeck,
              });
            }}
          >
            {validStepSizes.map((ss, idx) => (
              <option key={idx}>{ss}</option>
            ))}
          </select>
        </div>
        <div>
          <label>Number of steps where nobody gains a level: </label>
          <select
            value={`${props.params.deadzone_size}`}
            onChange={(evt) => {
              evt.preventDefault();
              const deadzoneSize = parseInt(evt.target.value, 10);

              updateSettings({
                deadzone_size: deadzoneSize,
              });
            }}
          >
            {Array(Math.max(maxSteps, props.params.deadzone_size))
              .fill(undefined)
              .map((_, idx) => (
                <option key={idx}>{idx}</option>
              ))}
          </select>
        </div>
        <div>
          <label>Number of steps for the attacking team to win: </label>
          <select
            value={`${props.params.num_steps_to_non_landlord_turnover}`}
            onChange={(evt) => {
              evt.preventDefault();
              const steps = parseInt(evt.target.value, 10);

              updateSettings({
                num_steps_to_non_landlord_turnover: steps,
              });
            }}
          >
            {Array(
              Math.max(
                maxSteps,
                props.params.num_steps_to_non_landlord_turnover
              )
            )
              .fill(undefined)
              .map((_, idx) => (
                <option key={idx + 1}>{idx + 1}</option>
              ))}
          </select>
        </div>
        <div>
          <label>Grant a bonus level for unexpectedly small team</label>{" "}
          <input
            id="small-team-bonus"
            type="checkbox"
            onChange={(evt) => {
              evt.preventDefault();
              updateSettings({
                bonus_level_policy: evt.target.checked
                  ? "BonusLevelForSmallerLandlordTeam"
                  : "NoBonusLevel",
              });
            }}
            checked={bonusEnabled}
          />
        </div>
      </div>
    </>
  );
};
