import * as React from "react";

function SvgTh(props) {
  return (
    <svg
      className="TH_svg__card"
      preserveAspectRatio="none"
      viewBox="-106 -164.5 212 329"
      {...props}
    >
      <symbol
        id="TH_svg__b"
        viewBox="-600 -600 1200 1200"
        preserveAspectRatio="xMinYMid"
      >
        <path
          d="M0-300c0-100 100-200 200-200s200 100 200 250C400 0 0 400 0 500 0 400-400 0-400-250c0-150 100-250 200-250S0-400 0-300z"
          fill="red"
        />
      </symbol>
      <symbol
        id="TH_svg__a"
        viewBox="-500 -500 1000 1000"
        preserveAspectRatio="xMinYMid"
      >
        <path
          d="M-260 430v-860M-50 0v-310a150 150 0 01300 0v620a150 150 0 01-300 0z"
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
      <use xlinkHref="#TH_svg__a" height={50} x={-104} y={-152.5} />
      <use xlinkHref="#TH_svg__b" height={41.827} x={-99.913} y={-97.5} />
      <use xlinkHref="#TH_svg__b" height={40} x={-59.001} y={-117.501} />
      <use xlinkHref="#TH_svg__b" height={40} x={19.001} y={-117.501} />
      <use xlinkHref="#TH_svg__b" height={40} x={-59.001} y={-52.5} />
      <use xlinkHref="#TH_svg__b" height={40} x={19.001} y={-52.5} />
      <use xlinkHref="#TH_svg__b" height={40} x={-20} y={-85} />
      <g transform="rotate(180)">
        <use xlinkHref="#TH_svg__a" height={50} x={-104} y={-152.5} />
        <use xlinkHref="#TH_svg__b" height={41.827} x={-99.913} y={-97.5} />
        <use xlinkHref="#TH_svg__b" height={40} x={-59.001} y={-117.501} />
        <use xlinkHref="#TH_svg__b" height={40} x={19.001} y={-117.501} />
        <use xlinkHref="#TH_svg__b" height={40} x={-59.001} y={-52.5} />
        <use xlinkHref="#TH_svg__b" height={40} x={19.001} y={-52.5} />
        <use xlinkHref="#TH_svg__b" height={40} x={-20} y={-85} />
      </g>
    </svg>
  );
}

export default SvgTh;
