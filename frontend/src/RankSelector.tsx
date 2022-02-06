import * as React from "react";

interface IProps {
  rank: string;
  onChangeRank: (newRank: string) => void;
}

// prettier-ignore
const allRanks = [
  '2', '3', '4', '5', '6', '7', '8',
  '9', '10', 'J', 'Q', 'K', 'A', 'NT'
]
const RankSelector = (props: IProps): JSX.Element => {
  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>): void => {
    if (e.target.value !== "") {
      props.onChangeRank(e.target.value);
    }
  };

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
      </label>
    </div>
  );
};

export default RankSelector;
