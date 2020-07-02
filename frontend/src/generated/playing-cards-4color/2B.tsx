import * as React from "react";

function Svg2B(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg
      className="2B_svg__card"
      preserveAspectRatio="none"
      viewBox="-106 -164.5 212 329"
      {...props}
    >
      <defs>
        <pattern
          id="2B_svg__a"
          width={6}
          height={6}
          patternUnits="userSpaceOnUse"
        >
          <path d="M3 0l3 3-3 3-3-3z" fill="red" />
        </pattern>
      </defs>
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
      <rect
        fill="url(#2B_svg__a)"
        width={180}
        height={300}
        x={-90}
        y={-150}
        rx={12}
        ry={12}
      />
    </svg>
  );
}

export default Svg2B;
