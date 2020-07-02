import * as React from "react";

import classNames from "classnames";
import InlineCard from "./InlineCard";
import { cardLookup } from "./util/cardHelpers";
import { SettingsContext } from "./AppStateProvider";

const Svg1B = React.lazy(
  async () => await import("./generated/playing-cards/1B")
);
const Svg2C = React.lazy(
  async () => await import("./generated/playing-cards/2C")
);
const Svg2J = React.lazy(
  async () => await import("./generated/playing-cards/2J")
);
const Svg3D = React.lazy(
  async () => await import("./generated/playing-cards/3D")
);
const Svg4C = React.lazy(
  async () => await import("./generated/playing-cards/4C")
);
const Svg4S = React.lazy(
  async () => await import("./generated/playing-cards/4S")
);
const Svg5H = React.lazy(
  async () => await import("./generated/playing-cards/5H")
);
const Svg6D = React.lazy(
  async () => await import("./generated/playing-cards/6D")
);
const Svg7C = React.lazy(
  async () => await import("./generated/playing-cards/7C")
);
const Svg7S = React.lazy(
  async () => await import("./generated/playing-cards/7S")
);
const Svg8H = React.lazy(
  async () => await import("./generated/playing-cards/8H")
);
const Svg9D = React.lazy(
  async () => await import("./generated/playing-cards/9D")
);
const SvgAc = React.lazy(
  async () => await import("./generated/playing-cards/Ac")
);
const SvgAs = React.lazy(
  async () => await import("./generated/playing-cards/As")
);
const SvgJh = React.lazy(
  async () => await import("./generated/playing-cards/Jh")
);
const SvgKd = React.lazy(
  async () => await import("./generated/playing-cards/Kd")
);
const SvgQc = React.lazy(
  async () => await import("./generated/playing-cards/Qc")
);
const SvgQs = React.lazy(
  async () => await import("./generated/playing-cards/Qs")
);
const SvgTh = React.lazy(
  async () => await import("./generated/playing-cards/Th")
);
const Svg1J = React.lazy(
  async () => await import("./generated/playing-cards/1J")
);
const Svg2D = React.lazy(
  async () => await import("./generated/playing-cards/2D")
);
const Svg2S = React.lazy(
  async () => await import("./generated/playing-cards/2S")
);
const Svg3H = React.lazy(
  async () => await import("./generated/playing-cards/3H")
);
const Svg4D = React.lazy(
  async () => await import("./generated/playing-cards/4D")
);
const Svg5C = React.lazy(
  async () => await import("./generated/playing-cards/5C")
);
const Svg5S = React.lazy(
  async () => await import("./generated/playing-cards/5S")
);
const Svg6H = React.lazy(
  async () => await import("./generated/playing-cards/6H")
);
const Svg7D = React.lazy(
  async () => await import("./generated/playing-cards/7D")
);
const Svg8C = React.lazy(
  async () => await import("./generated/playing-cards/8C")
);
const Svg8S = React.lazy(
  async () => await import("./generated/playing-cards/8S")
);
const Svg9H = React.lazy(
  async () => await import("./generated/playing-cards/9H")
);
const SvgAd = React.lazy(
  async () => await import("./generated/playing-cards/Ad")
);
const SvgJc = React.lazy(
  async () => await import("./generated/playing-cards/Jc")
);
const SvgJs = React.lazy(
  async () => await import("./generated/playing-cards/Js")
);
const SvgKh = React.lazy(
  async () => await import("./generated/playing-cards/Kh")
);
const SvgQd = React.lazy(
  async () => await import("./generated/playing-cards/Qd")
);
const SvgTc = React.lazy(
  async () => await import("./generated/playing-cards/Tc")
);
const SvgTs = React.lazy(
  async () => await import("./generated/playing-cards/Ts")
);
const Svg2H = React.lazy(
  async () => await import("./generated/playing-cards/2H")
);
const Svg3C = React.lazy(
  async () => await import("./generated/playing-cards/3C")
);
const Svg3S = React.lazy(
  async () => await import("./generated/playing-cards/3S")
);
const Svg4H = React.lazy(
  async () => await import("./generated/playing-cards/4H")
);
const Svg5D = React.lazy(
  async () => await import("./generated/playing-cards/5D")
);
const Svg6C = React.lazy(
  async () => await import("./generated/playing-cards/6C")
);
const Svg6S = React.lazy(
  async () => await import("./generated/playing-cards/6S")
);
const Svg7H = React.lazy(
  async () => await import("./generated/playing-cards/7H")
);
const Svg8D = React.lazy(
  async () => await import("./generated/playing-cards/8D")
);
const Svg9C = React.lazy(
  async () => await import("./generated/playing-cards/9C")
);
const Svg9S = React.lazy(
  async () => await import("./generated/playing-cards/9S")
);
const SvgAh = React.lazy(
  async () => await import("./generated/playing-cards/Ah")
);
const SvgJd = React.lazy(
  async () => await import("./generated/playing-cards/Jd")
);
const SvgKc = React.lazy(
  async () => await import("./generated/playing-cards/Kc")
);
const SvgKs = React.lazy(
  async () => await import("./generated/playing-cards/Ks")
);
const SvgQh = React.lazy(
  async () => await import("./generated/playing-cards/Qh")
);
const SvgTd = React.lazy(
  async () => await import("./generated/playing-cards/Td")
);
const Svg4Color2C = React.lazy(
  async () => await import("./generated/playing-cards-4color/2C")
);
const Svg4Color3D = React.lazy(
  async () => await import("./generated/playing-cards-4color/3D")
);
const Svg4Color4C = React.lazy(
  async () => await import("./generated/playing-cards-4color/4C")
);
const Svg4Color4S = React.lazy(
  async () => await import("./generated/playing-cards-4color/4S")
);
const Svg4Color5H = React.lazy(
  async () => await import("./generated/playing-cards-4color/5H")
);
const Svg4Color6D = React.lazy(
  async () => await import("./generated/playing-cards-4color/6D")
);
const Svg4Color7C = React.lazy(
  async () => await import("./generated/playing-cards-4color/7C")
);
const Svg4Color7S = React.lazy(
  async () => await import("./generated/playing-cards-4color/7S")
);
const Svg4Color8H = React.lazy(
  async () => await import("./generated/playing-cards-4color/8H")
);
const Svg4Color9D = React.lazy(
  async () => await import("./generated/playing-cards-4color/9D")
);
const Svg4ColorAc = React.lazy(
  async () => await import("./generated/playing-cards-4color/Ac")
);
const Svg4ColorAs = React.lazy(
  async () => await import("./generated/playing-cards-4color/As")
);
const Svg4ColorJh = React.lazy(
  async () => await import("./generated/playing-cards-4color/Jh")
);
const Svg4ColorKd = React.lazy(
  async () => await import("./generated/playing-cards-4color/Kd")
);
const Svg4ColorQc = React.lazy(
  async () => await import("./generated/playing-cards-4color/Qc")
);
const Svg4ColorQs = React.lazy(
  async () => await import("./generated/playing-cards-4color/Qs")
);
const Svg4ColorTh = React.lazy(
  async () => await import("./generated/playing-cards-4color/Th")
);
const Svg4Color2D = React.lazy(
  async () => await import("./generated/playing-cards-4color/2D")
);
const Svg4Color2S = React.lazy(
  async () => await import("./generated/playing-cards-4color/2S")
);
const Svg4Color3H = React.lazy(
  async () => await import("./generated/playing-cards-4color/3H")
);
const Svg4Color4D = React.lazy(
  async () => await import("./generated/playing-cards-4color/4D")
);
const Svg4Color5C = React.lazy(
  async () => await import("./generated/playing-cards-4color/5C")
);
const Svg4Color5S = React.lazy(
  async () => await import("./generated/playing-cards-4color/5S")
);
const Svg4Color6H = React.lazy(
  async () => await import("./generated/playing-cards-4color/6H")
);
const Svg4Color7D = React.lazy(
  async () => await import("./generated/playing-cards-4color/7D")
);
const Svg4Color8C = React.lazy(
  async () => await import("./generated/playing-cards-4color/8C")
);
const Svg4Color8S = React.lazy(
  async () => await import("./generated/playing-cards-4color/8S")
);
const Svg4Color9H = React.lazy(
  async () => await import("./generated/playing-cards-4color/9H")
);
const Svg4ColorAd = React.lazy(
  async () => await import("./generated/playing-cards-4color/Ad")
);
const Svg4ColorJc = React.lazy(
  async () => await import("./generated/playing-cards-4color/Jc")
);
const Svg4ColorJs = React.lazy(
  async () => await import("./generated/playing-cards-4color/Js")
);
const Svg4ColorKh = React.lazy(
  async () => await import("./generated/playing-cards-4color/Kh")
);
const Svg4ColorQd = React.lazy(
  async () => await import("./generated/playing-cards-4color/Qd")
);
const Svg4ColorTc = React.lazy(
  async () => await import("./generated/playing-cards-4color/Tc")
);
const Svg4ColorTs = React.lazy(
  async () => await import("./generated/playing-cards-4color/Ts")
);
const Svg4Color2H = React.lazy(
  async () => await import("./generated/playing-cards-4color/2H")
);
const Svg4Color3C = React.lazy(
  async () => await import("./generated/playing-cards-4color/3C")
);
const Svg4Color3S = React.lazy(
  async () => await import("./generated/playing-cards-4color/3S")
);
const Svg4Color4H = React.lazy(
  async () => await import("./generated/playing-cards-4color/4H")
);
const Svg4Color5D = React.lazy(
  async () => await import("./generated/playing-cards-4color/5D")
);
const Svg4Color6C = React.lazy(
  async () => await import("./generated/playing-cards-4color/6C")
);
const Svg4Color6S = React.lazy(
  async () => await import("./generated/playing-cards-4color/6S")
);
const Svg4Color7H = React.lazy(
  async () => await import("./generated/playing-cards-4color/7H")
);
const Svg4Color8D = React.lazy(
  async () => await import("./generated/playing-cards-4color/8D")
);
const Svg4Color9C = React.lazy(
  async () => await import("./generated/playing-cards-4color/9C")
);
const Svg4Color9S = React.lazy(
  async () => await import("./generated/playing-cards-4color/9S")
);
const Svg4ColorAh = React.lazy(
  async () => await import("./generated/playing-cards-4color/Ah")
);
const Svg4ColorJd = React.lazy(
  async () => await import("./generated/playing-cards-4color/Jd")
);
const Svg4ColorKc = React.lazy(
  async () => await import("./generated/playing-cards-4color/Kc")
);
const Svg4ColorKs = React.lazy(
  async () => await import("./generated/playing-cards-4color/Ks")
);
const Svg4ColorQh = React.lazy(
  async () => await import("./generated/playing-cards-4color/Qh")
);
const Svg4ColorTd = React.lazy(
  async () => await import("./generated/playing-cards-4color/Td")
);

const NormalCards: {
  [card: string]: React.FunctionComponent<React.SVGProps<SVGElement>>;
} = {
  "🃁": SvgAd,
  "🃎": SvgKd,
  "🃍": SvgQd,
  "🃋": SvgJd,
  "🃊": SvgTd,
  "🃉": Svg9D,
  "🃈": Svg8D,
  "🃇": Svg7D,
  "🃆": Svg6D,
  "🃅": Svg5D,
  "🃄": Svg4D,
  "🃃": Svg3D,
  "🃂": Svg2D,
  "🃑": SvgAc,
  "🃞": SvgKc,
  "🃝": SvgQc,
  "🃛": SvgJc,
  "🃚": SvgTc,
  "🃙": Svg9C,
  "🃘": Svg8C,
  "🃗": Svg7C,
  "🃖": Svg6C,
  "🃕": Svg5C,
  "🃔": Svg4C,
  "🃓": Svg3C,
  "🃒": Svg2C,
  "🂱": SvgAh,
  "🂾": SvgKh,
  "🂽": SvgQh,
  "🂻": SvgJh,
  "🂺": SvgTh,
  "🂹": Svg9H,
  "🂸": Svg8H,
  "🂷": Svg7H,
  "🂶": Svg6H,
  "🂵": Svg5H,
  "🂴": Svg4H,
  "🂳": Svg3H,
  "🂲": Svg2H,
  "🂡": SvgAs,
  "🂮": SvgKs,
  "🂭": SvgQs,
  "🂫": SvgJs,
  "🂪": SvgTs,
  "🂩": Svg9S,
  "🂨": Svg8S,
  "🂧": Svg7S,
  "🂦": Svg6S,
  "🂥": Svg5S,
  "🂤": Svg4S,
  "🂣": Svg3S,
  "🂢": Svg2S,
  "🃟": Svg2J,
  "🃏": Svg1J,
  "🂠": Svg1B,
};

const FourColorCards: {
  [card: string]: React.FunctionComponent<React.SVGProps<SVGElement>>;
} = {
  "🃁": Svg4ColorAd,
  "🃎": Svg4ColorKd,
  "🃍": Svg4ColorQd,
  "🃋": Svg4ColorJd,
  "🃊": Svg4ColorTd,
  "🃉": Svg4Color9D,
  "🃈": Svg4Color8D,
  "🃇": Svg4Color7D,
  "🃆": Svg4Color6D,
  "🃅": Svg4Color5D,
  "🃄": Svg4Color4D,
  "🃃": Svg4Color3D,
  "🃂": Svg4Color2D,
  "🃑": Svg4ColorAc,
  "🃞": Svg4ColorKc,
  "🃝": Svg4ColorQc,
  "🃛": Svg4ColorJc,
  "🃚": Svg4ColorTc,
  "🃙": Svg4Color9C,
  "🃘": Svg4Color8C,
  "🃗": Svg4Color7C,
  "🃖": Svg4Color6C,
  "🃕": Svg4Color5C,
  "🃔": Svg4Color4C,
  "🃓": Svg4Color3C,
  "🃒": Svg4Color2C,
  "🂱": Svg4ColorAh,
  "🂾": Svg4ColorKh,
  "🂽": Svg4ColorQh,
  "🂻": Svg4ColorJh,
  "🂺": Svg4ColorTh,
  "🂹": Svg4Color9H,
  "🂸": Svg4Color8H,
  "🂷": Svg4Color7H,
  "🂶": Svg4Color6H,
  "🂵": Svg4Color5H,
  "🂴": Svg4Color4H,
  "🂳": Svg4Color3H,
  "🂲": Svg4Color2H,
  "🂡": Svg4ColorAs,
  "🂮": Svg4ColorKs,
  "🂭": Svg4ColorQs,
  "🂫": Svg4ColorJs,
  "🂪": Svg4ColorTs,
  "🂩": Svg4Color9S,
  "🂨": Svg4Color8S,
  "🂧": Svg4Color7S,
  "🂦": Svg4Color6S,
  "🂥": Svg4Color5S,
  "🂤": Svg4Color4S,
  "🂣": Svg4Color3S,
  "🂢": Svg4Color2S,
  "🃟": Svg2J,
  "🃏": Svg1J,
  "🂠": Svg1B,
};

interface IProps {
  card: string;
  svgCards?: boolean;
  className?: string;
  onClick?: (event: React.MouseEvent) => void;
}

const Card = (props: IProps): JSX.Element => {
  const settings = React.useContext(SettingsContext);
  if (!(props.card in cardLookup)) {
    const nonSVG = (
      <span className={classNames("card", "unknown", props.className)}>
        {props.card}
      </span>
    );

    if (settings.svgCards) {
    } else {
      return nonSVG;
    }
    return (
      <React.Suspense fallback={nonSVG}>
        <span className={classNames("card", "svg", "unknown", props.className)}>
          <Svg1B height={120} />
        </span>
      </React.Suspense>
    );
  } else {
    const cardInfo = cardLookup[props.card];
    const nonSVG = (
      <span
        className={classNames("card", cardInfo.typ, props.className)}
        onClick={props.onClick}
      >
        <div className="card-label">
          <InlineCard card={props.card} />
        </div>
        {cardInfo.display_value}
      </span>
    );

    if (settings.svgCards) {
      return (
        <React.Suspense fallback={nonSVG}>
          <span
            className={classNames("card", "svg", cardInfo.typ, props.className)}
            onClick={props.onClick}
          >
            <div className="card-label">
              <InlineCard card={props.card} />
            </div>
            {React.createElement(
              (settings.fourColor ? FourColorCards : NormalCards)[props.card],
              {
                height: 120,
              }
            )}
          </span>
        </React.Suspense>
      );
    } else {
      return nonSVG;
    }
  }
};

export default Card;
