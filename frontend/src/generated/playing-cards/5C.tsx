import * as React from "react";

function Svg5C(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg
      className="5C_svg__card"
      preserveAspectRatio="none"
      viewBox="-106 -164.5 212 329"
      {...props}
    >
      <symbol
        id="5C_svg__b"
        viewBox="-600 -600 1200 1200"
        preserveAspectRatio="xMinYMid"
      >
        <path d="M30 150c5 235 55 250 100 350h-260c45-100 95-115 100-350a10 10 0 00-20 0 210 210 0 11-74-201 10 10 0 0014-14 230 230 0 11220 0 10 10 0 0014 14 210 210 0 11-74 201 10 10 0 00-20 0z" />
      </symbol>
      <symbol
        id="5C_svg__a"
        viewBox="-500 -500 1000 1000"
        preserveAspectRatio="xMinYMid"
      >
        <path
          d="M170-460h-345l-35 345s10-85 210-85c100 0 255 120 255 320S180 460-20 460s-235-175-235-175"
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
      <use xlinkHref="#5C_svg__a" height={50} x={-104} y={-152.5} />
      <use xlinkHref="#5C_svg__b" height={41.827} x={-99.913} y={-97.5} />
      <use xlinkHref="#5C_svg__b" height={40} x={-59.001} y={-117.551} />
      <use xlinkHref="#5C_svg__b" height={40} x={19.001} y={-117.551} />
      <use xlinkHref="#5C_svg__b" height={40} x={-20} y={-20} />
      <g transform="rotate(180)">
        <use xlinkHref="#5C_svg__a" height={50} x={-104} y={-152.5} />
        <use xlinkHref="#5C_svg__b" height={41.827} x={-99.913} y={-97.5} />
        <use xlinkHref="#5C_svg__b" height={40} x={-59.001} y={-117.551} />
        <use xlinkHref="#5C_svg__b" height={40} x={19.001} y={-117.551} />
      </g>
    </svg>
  );
}

export default Svg5C;
