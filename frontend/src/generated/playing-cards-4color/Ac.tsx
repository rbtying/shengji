import * as React from "react";

function SvgAc(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg
      className="AC_svg__card"
      preserveAspectRatio="none"
      viewBox="-106 -164.5 212 329"
      {...props}
    >
      <symbol
        id="AC_svg__b"
        viewBox="-500 -500 1000 1000"
        preserveAspectRatio="xMinYMid"
      >
        <path
          d="M-270 460h160m-90-10L0-460l200 910m-90 10h160m-390-330h240"
          stroke="green"
          strokeWidth={80}
          strokeLinecap="square"
          strokeMiterlimit={1.5}
          fill="none"
        />
      </symbol>
      <symbol
        id="AC_svg__a"
        viewBox="-600 -600 1200 1200"
        preserveAspectRatio="xMinYMid"
      >
        <path
          d="M30 150c5 235 55 250 100 350h-260c45-100 95-115 100-350a10 10 0 00-20 0 210 210 0 11-74-201 10 10 0 0014-14 230 230 0 11220 0 10 10 0 0014 14 210 210 0 11-74 201 10 10 0 00-20 0z"
          fill="green"
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
      <use xlinkHref="#AC_svg__a" height={40} x={-20} y={-20} />
      <use xlinkHref="#AC_svg__b" height={50} x={-104} y={-152.5} />
      <use xlinkHref="#AC_svg__a" height={41.827} x={-99.913} y={-97.5} />
      <g transform="rotate(180)">
        <use xlinkHref="#AC_svg__b" height={50} x={-104} y={-152.5} />
        <use xlinkHref="#AC_svg__a" height={41.827} x={-99.913} y={-97.5} />
      </g>
    </svg>
  );
}

export default SvgAc;
