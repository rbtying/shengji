import * as React from "react";

import classNames from "classnames";
import memoize from "./memoize";
import InlineCard from "./InlineCard";
import { cardLookup } from "./util/cardHelpers";
import { SettingsContext } from "./AppStateProvider";

const SvgCard = React.lazy(async () => await import("./SvgCard"));

interface IProps {
  card: string;
  smaller?: boolean;
  className?: string;
  onClick?: (event: React.MouseEvent) => void;
}

const Card = (props: IProps): JSX.Element => {
  const settings = React.useContext(SettingsContext);
  if (!(props.card in cardLookup)) {
    const nonSVG = (
      <span className={classNames("card", "unknown", props.className)}>
        <CardCanvas
          card={props.card}
          height={props.smaller ? 95 : 120}
          suit={classNames("unknown", settings.fourColor ? "four-color" : null)}
        />
      </span>
    );

    if (settings.svgCards) {
      return (
        <React.Suspense fallback={nonSVG}>
          <span
            className={classNames("card", "svg", "unknown", props.className)}
          >
            <SvgCard
              fourColor={settings.fourColor}
              smaller={props.smaller}
              card={"🂠"}
            />
          </span>
        </React.Suspense>
      );
    } else {
      return nonSVG;
    }
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
        <CardCanvas
          card={cardInfo.display_value}
          height={props.smaller ? 95 : 120}
          suit={classNames(
            cardInfo.typ,
            settings.fourColor ? "four-color" : null
          )}
        />
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
            <SvgCard
              fourColor={settings.fourColor}
              smaller={props.smaller}
              card={props.card}
            />
          </span>
        </React.Suspense>
      );
    } else {
      return nonSVG;
    }
  }
};

const computeCanvasBounds = (font: string): [number, number, number] => {
  const c = document.createElement("canvas");
  c.style.display = "none";
  document.body.appendChild(c);
  const ctx = c.getContext("2d");
  ctx.font = font;
  const text = "🂠";
  const textMetrics = ctx.measureText(text);
  const width =
    textMetrics.actualBoundingBoxLeft + textMetrics.actualBoundingBoxRight;
  const height =
    textMetrics.actualBoundingBoxAscent + textMetrics.actualBoundingBoxDescent;
  document.body.removeChild(c);
  return [width, height, textMetrics.actualBoundingBoxDescent];
};

const computeSuitColor = (suit: string): string => {
  const c = document.createElement("span");
  c.className = suit;
  c.style.display = "none";
  document.body.appendChild(c);
  const color = getComputedStyle(c).color;
  document.body.removeChild(c);
  return color;
};

const cardBoundsCache: { [font: string]: () => [number, number, number] } = {};
const suitColorCache: { [suit: string]: () => string } = {};

interface ICardCanvasProps {
  card: string;
  height: number;
  suit: string;
}

const CardCanvas = (props: ICardCanvasProps): JSX.Element => {
  const canvasRef = React.useRef<HTMLCanvasElement>(null);
  const dpr =
    (window.devicePixelRatio !== undefined ? window.devicePixelRatio : 1) * 1.5;
  const font = `${props.height * dpr}px solid`;
  if (!(font in cardBoundsCache)) {
    cardBoundsCache[font] = memoize(() => computeCanvasBounds(font));
  }
  if (!(props.suit in suitColorCache)) {
    suitColorCache[props.suit] = memoize(() => computeSuitColor(props.suit));
  }
  const [width, height, offset] = cardBoundsCache[font]();
  const style = suitColorCache[props.suit]();

  React.useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");
    ctx.font = font;
    ctx.textAlign = "center";
    ctx.textBaseline = "bottom";
    ctx.clearRect(0, 0, width + 2, height + 2);
    ctx.fillStyle = "#fff";
    ctx.fillRect(2, 2, width, height);
    ctx.fillStyle = style;
    ctx.fillText(props.card, width / 2 + 1, height + offset - 3 * dpr);
  });

  return (
    <canvas
      ref={canvasRef}
      width={width + 2}
      height={height + 2}
      style={{
        width: width / dpr + 2,
        height: height / dpr + 2,
      }}
    />
  );
};

export default Card;
