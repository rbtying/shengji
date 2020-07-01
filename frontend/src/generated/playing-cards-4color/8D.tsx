import * as React from "react";

function Svg8D(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg
      className="8D_svg__card"
      preserveAspectRatio="none"
      viewBox="-106 -164.5 212 329"
      {...props}
    >
      <symbol
        id="8D_svg__b"
        viewBox="-600 -600 1200 1200"
        preserveAspectRatio="xMinYMid"
      >
        <path
          d="M-400 0C-350 0 0-450 0-500 0-450 350 0 400 0 350 0 0 450 0 500 0 450-350 0-400 0z"
          fill="#00f"
        />
      </symbol>
      <symbol
        id="8D_svg__a"
        viewBox="-500 -500 1000 1000"
        preserveAspectRatio="xMinYMid"
      >
        <path
          d="M-1-50a205 205 0 112 0h-2a255 255 0 102 0z"
          stroke="#00f"
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
      <use xlinkHref="#8D_svg__a" height={50} x={-104} y={-152.5} />
      <use xlinkHref="#8D_svg__b" height={41.827} x={-99.913} y={-97.5} />
      <use xlinkHref="#8D_svg__b" height={40} x={-59.001} y={-117.501} />
      <use xlinkHref="#8D_svg__b" height={40} x={19.001} y={-117.501} />
      <use xlinkHref="#8D_svg__b" height={40} x={-20} y={-68.75} />
      <use xlinkHref="#8D_svg__b" height={40} x={-59.001} y={-20} />
      <use xlinkHref="#8D_svg__b" height={40} x={19.001} y={-20} />
      <g transform="rotate(180)">
        <use xlinkHref="#8D_svg__a" height={50} x={-104} y={-152.5} />
        <use xlinkHref="#8D_svg__b" height={41.827} x={-99.913} y={-97.5} />
        <use xlinkHref="#8D_svg__b" height={40} x={-59.001} y={-117.501} />
        <use xlinkHref="#8D_svg__b" height={40} x={19.001} y={-117.501} />
        <use xlinkHref="#8D_svg__b" height={40} x={-20} y={-68.75} />
      </g>
    </svg>
  );
}

export default Svg8D;
