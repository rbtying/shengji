import * as React from 'react';

type Props = {
  rank: string;
  onChangeRank: (newRank: string) => void;
};

// prettier-ignore
const allRanks = [
  '2', '3', '4', '5', '6', '7', '8',
  '9', '10', 'J', 'Q', 'K', 'A'
]
const RankSelector = (props: Props) => {
  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    if (e.target.value !== '') {
      props.onChangeRank(e.target.value);
    }
  };

  return (
    <div className="rank-picker">
      <label>
        Your rank:{' '}
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
