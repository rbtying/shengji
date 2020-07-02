import * as React from "react";

function Svg2H(props) {
  return (
    <svg
      className="2H_svg__card"
      preserveAspectRatio="none"
      viewBox="-106 -164.5 212 329"
      {...props}
    >
      <symbol
        id="2H_svg__b"
        viewBox="-600 -600 1200 1200"
        preserveAspectRatio="xMinYMid"
      >
        <path
          d="M0-300c0-100 100-200 200-200s200 100 200 250C400 0 0 400 0 500 0 400-400 0-400-250c0-150 100-250 200-250S0-400 0-300z"
          fill="red"
        />
      </symbol>
      <symbol
        id="2H_svg__a"
        viewBox="-500 -500 1000 1000"
        preserveAspectRatio="xMinYMid"
      >
        <path
          d="M-225-225c-20-40 25-235 225-235s225 135 225 235c0 200-450 385-450 685h450V300"
          stroke="red"
          strokeWidth={80}
          strokeLinecap="square"
          strokeMiterlimit={1.5}
          fill="none"
        />
      </symbol>
      <rect
        width={211}
        height={328}
        x={-105.5}
        y={-164}
        rx={12}
        ry={12}
        fill="#fff"
        stroke="#000"
      />
      <use xlinkHref="#2H_svg__a" height={50} x={-104} y={-152.5} />
      <use xlinkHref="#2H_svg__b" height={41.827} x={-99.913} y={-97.5} />
      <use xlinkHref="#2H_svg__b" height={40} x={-20} y={-117.501} />
      <g transform="rotate(180)">
        <use xlinkHref="#2H_svg__a" height={50} x={-104} y={-152.5} />
        <use xlinkHref="#2H_svg__b" height={41.827} x={-99.913} y={-97.5} />
        <use xlinkHref="#2H_svg__b" height={40} x={-20} y={-117.501} />
      </g>
    </svg>
  );
}

export default Svg2H;
