import * as React from "react";

interface IProps {
  width?: string;
}
const BarChart = ({ width = "100%" }: IProps): JSX.Element => (
  <svg
    focusable="false"
    role="img"
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 16 16"
    fill="currentColor"
    width={width}
  >
    <rect width="4" height="5" x="1" y="10" rx="1" />
    <rect width="4" height="9" x="6" y="6" rx="1" />
    <rect width="4" height="14" x="11" y="1" rx="1" />
  </svg>
);

export default BarChart;
