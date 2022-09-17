import * as React from "react";

interface IProps {
  autodrawSpeedMs: number;
  onChange: (newAutodrawSpeedMs: number) => void;
}

const AutodrawSpeedSelector = (props: IProps): JSX.Element => {
  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>): void => {
    const newAutodrawSpeedMs = parseInt(e.target.value);
    props.onChange(newAutodrawSpeedMs);
  };

  return (
    <div className="autodraw-speed-picker">
      <label>
        Autodraw speed:{" "}
        <select
          value={props.autodrawSpeedMs === null ? "" : props.autodrawSpeedMs}
          onChange={handleChange}
        >
          <option value="250">default</option>
          <option value="500">slow</option>
          <option value="10">fast</option>
        </select>
      </label>
    </div>
  );
};

export default AutodrawSpeedSelector;
