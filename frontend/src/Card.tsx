import * as React from "react";

import classNames from "classnames";
import memoize from "./memoize";
import InlineCard from "./InlineCard";
import { cardLookup } from "./util/cardHelpers";
import { SettingsContext } from "./AppStateProvider";
import { ISuitOverrides } from "./state/Settings";
import { Trump } from "./gen-types";
import WasmContext from "./WasmContext";

const SvgCard = React.lazy(async () => await import("./SvgCard"));

interface IProps {
  card: string;
  trump: Trump;
  collapseRight?: boolean;
  smaller?: boolean;
  className?: string;
  onClick?: (event: React.MouseEvent) => void;
  onMouseEnter?: (event: React.MouseEvent) => void;
  onMouseLeave?: (event: React.MouseEvent) => void;
}

const Card = (props: IProps): JSX.Element => {
  const settings = React.useContext(SettingsContext);
  const { getCardInfo } = React.useContext(WasmContext);
  const height = props.smaller ? 95 : 120;
  const bounds = getCardBounds(height);

  if (!(props.card in cardLookup)) {
    const nonSVG = (
      <div
        className={classNames("card", "unknown", props.className)}
        style={{
          marginRight: props.collapseRight ? `-${bounds.width * 0.6}px` : "0",
        }}
      >
        <CardCanvas
          card={props.card}
          height={height}
          suit={classNames(
            "unknown",
            settings.fourColor ? "four-color" : null,
            settings.darkMode ? "dark-mode" : null
          )}
          backgroundColor={settings.darkMode ? "#000" : "#fff"}
        />
      </div>
    );

    if (settings.svgCards) {
      return (
        <React.Suspense fallback={nonSVG}>
          <div
            className={classNames("card", "svg", "unknown", props.className)}
            style={{
              marginRight: props.collapseRight
                ? `-${bounds.width * 0.6}px`
                : "0",
            }}
          >
            <SvgCard
              fourColor={settings.fourColor}
              height={height}
              card={"🂠"}
            />
          </div>
        </React.Suspense>
      );
    } else {
      return nonSVG;
    }
  } else {
    const cardInfo = cardLookup[props.card];
    const extraInfo = getCardInfo({ card: props.card, trump: props.trump });
    const label = (offset: number): JSX.Element => (
      <div className="card-label" style={{ bottom: `${offset}px` }}>
        <InlineCard card={props.card} />
      </div>
    );
    const icon = (offset: number): JSX.Element => (
      <div className="card-icon" style={{ bottom: `${offset}px` }}>
        {extraInfo.effective_suit === "Trump" && settings.trumpCardIcon}
        {extraInfo.points > 0 && settings.pointCardIcon}
      </div>
    );
    const nonSVG = (
      <div
        className={classNames("card", cardInfo.typ, props.className)}
        onClick={props.onClick}
        onMouseEnter={props.onMouseEnter}
        onMouseLeave={props.onMouseLeave}
        style={{
          marginRight: props.collapseRight ? `-${bounds.width * 0.6}px` : "0",
        }}
      >
        {label(bounds.height / 10)}
        {icon(bounds.height)}
        <CardCanvas
          card={cardInfo.display_value}
          height={height}
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
      </div>
    );

    if (settings.svgCards) {
      return (
        <React.Suspense fallback={nonSVG}>
          <div
            className={classNames("card", "svg", cardInfo.typ, props.className)}
            onClick={props.onClick}
            onMouseEnter={props.onMouseEnter}
            onMouseLeave={props.onMouseLeave}
            style={{
              marginRight: props.collapseRight
                ? `-${bounds.width * 0.6}px`
                : "0",
            }}
          >
            {label(height / 10)}
            {icon(height)}
            <SvgCard
              fourColor={settings.fourColor}
              height={height}
              card={props.card}
            />
          </div>
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
  const c = document.createElement("div");
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

const getCardBounds = (
  height: number
): { metrics: TextMetrics; height: number; width: number } => {
  const font = `${height}px solid`;
  if (!(font in cardBoundsCache)) {
    cardBoundsCache[font] = memoize(() => computeCanvasBounds(font, 1));
  }
  const textMetrics = cardBoundsCache[font]();

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
  return {
    metrics: textMetrics,
    height: effectiveHeight,
    width: effectiveWidth,
  };
};

const CardCanvas = (props: ICardCanvasProps): JSX.Element => {
  if (!(props.suit in suitColorCache)) {
    suitColorCache[props.suit] = memoize(() => computeSuitColor(props.suit));
  }
  const { metrics, width, height } = getCardBounds(props.height);
  const style = suitColorCache[props.suit]();
  return (
    <svg
      focusable="false"
      role="img"
      xmlns="http://www.w3.org/2000/svg"
      height={height}
      width={width}
    >
      <rect
        fill={
          props.backgroundColor !== undefined ? props.backgroundColor : "#fff"
        }
        x={metrics.actualBoundingBoxLeft}
        y={0}
        width={metrics.width - 2}
        height={height}
      ></rect>
      <text
        fill={props.colorOverride !== undefined ? props.colorOverride : style}
        fontSize={`${props.height}px`}
        textLength={`${width}px`}
        x={Math.min(metrics.actualBoundingBoxLeft, 0) + 1}
        y={height - metrics.actualBoundingBoxDescent - 1}
      >
        {props.card}
      </text>
    </svg>
  );
};

export default Card;
