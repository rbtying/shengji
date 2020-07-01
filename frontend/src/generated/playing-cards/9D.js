import * as React from "react";

function Svg9D(props) {
  return (
    <svg
      className="9D_svg__card"
      preserveAspectRatio="none"
      viewBox="-106 -164.5 212 329"
      {...props}
    >
      <symbol
        id="9D_svg__b"
        viewBox="-600 -600 1200 1200"
        preserveAspectRatio="xMinYMid"
      >
        <path
          d="M-400 0C-350 0 0-450 0-500 0-450 350 0 400 0 350 0 0 450 0 500 0 450-350 0-400 0z"
          fill="red"
        />
      </symbol>
      <symbol
        id="9D_svg__a"
        viewBox="-500 -500 1000 1000"
        preserveAspectRatio="xMinYMid"
      >
        <path
          d="M250-100a250 250 0 01-500 0v-110a250 250 0 01500 0v420A250 250 0 010 460c-150 0-180-60-200-85"
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
      <use xlinkHref="#9D_svg__a" height={50} x={-104} y={-152.5} />
      <use xlinkHref="#9D_svg__b" height={41.827} x={-99.913} y={-97.5} />
      <use xlinkHref="#9D_svg__b" height={40} x={-59.001} y={-117.501} />
      <use xlinkHref="#9D_svg__b" height={40} x={19.001} y={-117.501} />
      <use xlinkHref="#9D_svg__b" height={40} x={-59.001} y={-52.5} />
      <use xlinkHref="#9D_svg__b" height={40} x={19.001} y={-52.5} />
      <use xlinkHref="#9D_svg__b" height={40} x={-20} y={-20} />
      <g transform="rotate(180)">
        <use xlinkHref="#9D_svg__a" height={50} x={-104} y={-152.5} />
        <use xlinkHref="#9D_svg__b" height={41.827} x={-99.913} y={-97.5} />
        <use xlinkHref="#9D_svg__b" height={40} x={-59.001} y={-117.501} />
        <use xlinkHref="#9D_svg__b" height={40} x={19.001} y={-117.501} />
        <use xlinkHref="#9D_svg__b" height={40} x={-59.001} y={-52.5} />
        <use xlinkHref="#9D_svg__b" height={40} x={19.001} y={-52.5} />
      </g>
    </svg>
  );
}

export default Svg9D;
