import * as React from "react";

import classNames from "classnames";
import memoize from "./memoize";
import InlineCard from "./InlineCard";
import { cardLookup } from "./util/cardHelpers";
import { SettingsContext } from "./AppStateProvider";
import { ISuitOverrides } from "./state/Settings";

const SvgCard = React.lazy(async () => await import("./SvgCard"));

interface IProps {
  card: string;
  smaller?: boolean;
  className?: string;
  onClick?: (event: React.MouseEvent) => void;
  onMouseEnter?: (event: React.MouseEvent) => void;
  onMouseLeave?: (event: React.MouseEvent) => void;
}

const Card = (props: IProps): JSX.Element => {
  const settings = React.useContext(SettingsContext);
  if (!(props.card in cardLookup)) {
    const nonSVG = (
      <span className={classNames("card", "unknown", props.className)}>
        <CardCanvas
          card={props.card}
          height={props.smaller ? 95 : 120}
          suit={classNames(
            "unknown",
            settings.fourColor ? "four-color" : null,
            settings.darkMode ? "dark-mode" : null
          )}
          backgroundColor={settings.darkMode ? "#000" : "#fff"}
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
        onMouseEnter={props.onMouseEnter}
        onMouseLeave={props.onMouseLeave}
      >
        <div className="card-label">
          <InlineCard card={props.card} />
        </div>
        <CardCanvas
          card={cardInfo.display_value}
          height={props.smaller ? 95 : 120}
          suit={classNames(
            cardInfo.typ,
            settings.fourColor ? "four-color" : null,
            settings.darkMode ? "dark-mode" : null
          )}
          colorOverride={
            settings.suitColorOverrides[cardInfo.typ as keyof ISuitOverrides]
          }
          backgroundColor={settings.darkMode ? "#000" : "#fff"}
        />
      </span>
    );

    if (settings.svgCards) {
      return (
        <React.Suspense fallback={nonSVG}>
          <span
            className={classNames("card", "svg", cardInfo.typ, props.className)}
            onClick={props.onClick}
            onMouseEnter={props.onMouseEnter}
            onMouseLeave={props.onMouseLeave}
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

const computeCanvasBounds = (font: string, dpr: number): TextMetrics => {
  const c = document.createElement("canvas");
  c.style.display = "none";
  document.body.appendChild(c);
  const ctx = c.getContext("2d");
  ctx.scale(dpr, dpr);
  ctx.font = font;
  const text = "🂠";
  const textMetrics = ctx.measureText(text);
  document.body.removeChild(c);
  return textMetrics;
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

const cardBoundsCache: { [font: string]: () => TextMetrics } = {};
const suitColorCache: { [suit: string]: () => string } = {};

interface ICardCanvasProps {
  card: string;
  height: number;
  suit: string;
  backgroundColor?: string;
  colorOverride?: string;
}

const CardCanvas = (props: ICardCanvasProps): JSX.Element => {
  const font = `${props.height}px solid`;
  if (!(font in cardBoundsCache)) {
    cardBoundsCache[font] = memoize(() => computeCanvasBounds(font, 1));
  }
  if (!(props.suit in suitColorCache)) {
    suitColorCache[props.suit] = memoize(() => computeSuitColor(props.suit));
  }
  const textMetrics = cardBoundsCache[font]();
  const style = suitColorCache[props.suit]();

  const effectiveHeight = Math.round(
    textMetrics.actualBoundingBoxAscent +
      textMetrics.actualBoundingBoxDescent +
      2
  );
  const effectiveWidth = Math.round(
    textMetrics.actualBoundingBoxRight +
      Math.min(textMetrics.actualBoundingBoxLeft, 0) +
      2
  );
  return (
    <svg
      focusable="false"
      role="img"
      xmlns="http://www.w3.org/2000/svg"
      height={effectiveHeight}
      width={effectiveWidth}
    >
      <rect
        fill={
          props.backgroundColor !== undefined ? props.backgroundColor : "#fff"
        }
        x={textMetrics.actualBoundingBoxLeft}
        y={0}
        width={textMetrics.width - 2}
        height={effectiveHeight}
      ></rect>
      <text
        fill={props.colorOverride !== undefined ? props.colorOverride : style}
        fontSize={`${props.height}px`}
        textLength={`${textMetrics.width}px`}
        x={Math.min(textMetrics.actualBoundingBoxLeft, 0) + 1}
        y={effectiveHeight - textMetrics.actualBoundingBoxDescent - 1}
      >
        {props.card}
      </text>
    </svg>
  );
};

export default Card;
