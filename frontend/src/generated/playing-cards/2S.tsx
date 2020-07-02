import * as React from "react";

function Svg2S(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg
      className="2S_svg__card"
      preserveAspectRatio="none"
      viewBox="-106 -164.5 212 329"
      {...props}
    >
      <symbol
        id="2S_svg__b"
        viewBox="-600 -600 1200 1200"
        preserveAspectRatio="xMinYMid"
      >
        <path d="M0-500c100 250 355 400 355 685a150 150 0 01-300 0 10 10 0 00-20 0c0 200 50 215 95 315h-260c45-100 95-115 95-315a10 10 0 00-20 0 150 150 0 01-300 0c0-285 255-435 355-685z" />
      </symbol>
      <symbol
        id="2S_svg__a"
        viewBox="-500 -500 1000 1000"
        preserveAspectRatio="xMinYMid"
      >
        <path
          d="M-225-225c-20-40 25-235 225-235s225 135 225 235c0 200-450 385-450 685h450V300"
          stroke="#000"
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
      <use xlinkHref="#2S_svg__a" height={50} x={-104} y={-152.5} />
      <use xlinkHref="#2S_svg__b" height={41.827} x={-99.913} y={-97.5} />
      <use xlinkHref="#2S_svg__b" height={40} x={-20} y={-117.501} />
      <g transform="rotate(180)">
        <use xlinkHref="#2S_svg__a" height={50} x={-104} y={-152.5} />
        <use xlinkHref="#2S_svg__b" height={41.827} x={-99.913} y={-97.5} />
        <use xlinkHref="#2S_svg__b" height={40} x={-20} y={-117.501} />
      </g>
    </svg>
  );
}

export default Svg2S;
