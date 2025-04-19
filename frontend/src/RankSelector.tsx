import * as React from "react";

import type { JSX } from "react";

interface IProps {
  rank: string;
  metaRank: number;
  onChangeRank: (newRank: string) => void;
  onChangeMetaRank: (newMetaRank: number) => void;
}

// prettier-ignore
const allRanks = [
  '2', '3', '4', '5', '6', '7', '8',
  '9', '10', 'J', 'Q', 'K', 'A', 'NT'
]
const RankSelector = (props: IProps): JSX.Element => {
  const [showMetaRank, setShowMetaRank] = React.useState<boolean>(false);
  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>): void => {
    if (e.target.value !== "") {
      props.onChangeRank(e.target.value);
    }
  };
  const handleMetaChange = (e: React.ChangeEvent<HTMLSelectElement>): void => {
    if (e.target.value !== "") {
      const v = parseInt(e.target.value, 10);
      props.onChangeMetaRank(v);
    }
  };

  const metaranks = [];

  if (props.metaRank > 0) {
    for (let i = 1; i <= props.metaRank + 3; i++) {
      metaranks.push(i);
    }
  } else {
    metaranks.push(props.metaRank);
    metaranks.push(1);
  }

  return (
    <div className="rank-picker">
      <label>
        Your rank:{" "}
        <select value={props.rank} onChange={handleChange}>
          {allRanks.map((rank) => (
            <option value={rank} key={rank}>
              {rank}
            </option>
          ))}
        </select>
        <input
          type="checkbox"
          checked={showMetaRank}
          onChange={() => setShowMetaRank(!showMetaRank)}
          title="show meta-rank"
        />
        {showMetaRank && (
          <select value={props.metaRank} onChange={handleMetaChange}>
            {metaranks.map((metarank) => (
              <option value={metarank} key={metarank}>
                {metarank}
              </option>
            ))}
          </select>
        )}
      </label>
    </div>
  );
};

export default RankSelector;
