import * as React from "react";

import classNames from "classnames";
import memoize from "./memoize";
import InlineCard from "./InlineCard";
import { cardLookup } from "./util/cardHelpers";
import { SettingsContext } from "./AppStateProvider";
import { ISuitOverrides } from "./state/Settings";
import { Trump, CardInfo } from "./gen-types";
import { useEngine } from "./useEngine";
import {
  cardInfoCache,
  getTrumpKey,
  prefillCardInfoCache,
  getPrefillPromise,
} from "./util/cachePrefill";

import type { JSX } from "react";

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
  const engine = useEngine();
  const [cardInfo, setCardInfo] = React.useState<CardInfo | null>(null);
  const [isLoading, setIsLoading] = React.useState<boolean>(false);
  const height = props.smaller ? 95 : 120;
  const bounds = getCardBounds(height);

  // Create a cache key for the card info based on card and trump
  const cacheKey = `${props.card}_${getTrumpKey(props.trump)}`;

  React.useEffect(() => {
    // Only load card info if the card is in the lookup
    if (!(props.card in cardLookup)) {
      return;
    }

    // Check cache first
    if (cacheKey in cardInfoCache) {
      setCardInfo(cardInfoCache[cacheKey]);
      setIsLoading(false);
      return;
    }

    setIsLoading(true);

    // Check if a prefill is already in progress for this trump
    const existingPrefillPromise = getPrefillPromise(props.trump);
    if (existingPrefillPromise) {
      // Wait for existing prefill
      existingPrefillPromise
        .then(() => {
          // Check if our card is now cached
          if (cacheKey in cardInfoCache) {
            setCardInfo(cardInfoCache[cacheKey]);
            setIsLoading(false);
          } else {
            // If still not cached after prefill, something went wrong
            console.error(
              `Card ${props.card} not in cache after prefill completed`,
            );
            const staticInfo = cardLookup[props.card];
            setCardInfo({
              suit: null,
              effective_suit: "Unknown" as any,
              value: staticInfo.value || props.card,
              display_value: staticInfo.display_value || props.card,
              typ: staticInfo.typ || props.card,
              number: staticInfo.number || null,
              points: staticInfo.points || 0,
            });
            setIsLoading(false);
          }
        })
        .catch((error) => {
          console.error("Failed to wait for prefill:", error);
          setIsLoading(false);
        });
      return;
    }

    // Check if we should trigger a full prefill for this trump
    const trumpKey = getTrumpKey(props.trump);

    // Count how many cards are cached for this trump
    const cachedCount = Object.keys(cardInfoCache).filter((key) =>
      key.endsWith(`_${trumpKey}`),
    ).length;

    // If we have very few cached cards for this trump, prefill everything
    if (cachedCount < 5) {
      // Trigger full prefill for uncached trump

      // Start the prefill and wait for it
      prefillCardInfoCache(engine, props.trump)
        .then(() => {
          // Check if our card is now cached
          if (cacheKey in cardInfoCache) {
            setCardInfo(cardInfoCache[cacheKey]);
            setIsLoading(false);
          } else {
            // Fallback if card still not in cache
            const staticInfo = cardLookup[props.card];
            setCardInfo({
              suit: null,
              effective_suit: "Unknown" as any,
              value: staticInfo.value || props.card,
              display_value: staticInfo.display_value || props.card,
              typ: staticInfo.typ || props.card,
              number: staticInfo.number || null,
              points: staticInfo.points || 0,
            });
            setIsLoading(false);
          }
        })
        .catch((error) => {
          console.error("Failed to prefill cache:", error);
          // Fallback on error
          const staticInfo = cardLookup[props.card];
          setCardInfo({
            suit: null,
            effective_suit: "Unknown" as any,
            value: staticInfo.value || props.card,
            display_value: staticInfo.display_value || props.card,
            typ: staticInfo.typ || props.card,
            number: staticInfo.number || null,
            points: staticInfo.points || 0,
          });
          setIsLoading(false);
        });
      return;
    }

    // Only make individual request if no prefill is needed
    engine
      .batchGetCardInfo({
        requests: [
          {
            card: props.card,
            trump: props.trump,
          },
        ],
      })
      .then((response) => {
        if (!response || !response.results || response.results.length === 0) {
          console.error("Invalid response from batchGetCardInfo:", response);
          // Fallback to basic info from static lookup
          const staticInfo = cardLookup[props.card];
          setCardInfo({
            suit: null,
            effective_suit: "Unknown" as any,
            value: staticInfo.value || props.card,
            display_value: staticInfo.display_value || props.card,
            typ: staticInfo.typ || props.card,
            number: staticInfo.number || null,
            points: staticInfo.points || 0,
          });
          setIsLoading(false);
          return;
        }
        const info = response.results[0];
        if (!info) {
          console.error("Card info is undefined in response:", response);
          // Fallback to basic info from static lookup
          const staticInfo = cardLookup[props.card];
          setCardInfo({
            suit: null,
            effective_suit: "Unknown" as any,
            value: staticInfo.value || props.card,
            display_value: staticInfo.display_value || props.card,
            typ: staticInfo.typ || props.card,
            number: staticInfo.number || null,
            points: staticInfo.points || 0,
          });
          setIsLoading(false);
          return;
        }
        // Cache the result with the trump-specific key
        cardInfoCache[cacheKey] = info;
        setCardInfo(info);
        setIsLoading(false);
      })
      .catch((error) => {
        console.error("Error getting card info:", error);
        console.error("Error stack:", error.stack);
        // Fallback to basic info from static lookup
        const staticInfo = cardLookup[props.card];
        setCardInfo({
          suit: null,
          effective_suit: "Unknown" as any,
          value: staticInfo.value || props.card,
          display_value: staticInfo.display_value || props.card,
          typ: staticInfo.typ || props.card,
          number: staticInfo.number || null,
          points: staticInfo.points || 0,
        });
        setIsLoading(false);
      });
  }, [cacheKey, props.card, props.trump, engine]);

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
            settings.darkMode ? "dark-mode" : null,
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
    const staticCardInfo = cardLookup[props.card];

    const label = (offset: number): JSX.Element | null => {
      if (isLoading || !cardInfo) return null;
      return (
        <div className="card-label" style={{ bottom: `${offset}px` }}>
          <InlineCard card={props.card} />
        </div>
      );
    };

    const icon = (offset: number): JSX.Element | null => {
      if (isLoading || !cardInfo) return null;
      return (
        <div className="card-icon" style={{ bottom: `${offset}px` }}>
          {cardInfo.effective_suit === "Trump" && settings.trumpCardIcon}
          {cardInfo.points > 0 && settings.pointCardIcon}
        </div>
      );
    };

    const nonSVG = (
      <div
        className={classNames(
          "card",
          staticCardInfo.typ,
          props.className,
          isLoading ? "loading" : null,
        )}
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
          card={staticCardInfo.display_value}
          height={height}
          suit={classNames(
            staticCardInfo.typ,
            settings.fourColor ? "four-color" : null,
            settings.darkMode ? "dark-mode" : null,
          )}
          colorOverride={
            settings.suitColorOverrides[
              staticCardInfo.typ as keyof ISuitOverrides
            ]
          }
          backgroundColor={settings.darkMode ? "#000" : "#fff"}
        />
      </div>
    );

    if (settings.svgCards) {
      return (
        <React.Suspense fallback={nonSVG}>
          <div
            className={classNames(
              "card",
              "svg",
              staticCardInfo.typ,
              props.className,
            )}
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
  if (ctx === null) {
    throw new Error("Could not get 2d context");
  }
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
  height: number,
): { metrics: TextMetrics; height: number; width: number } => {
  const font = `${height}px solid`;
  if (!(font in cardBoundsCache)) {
    cardBoundsCache[font] = memoize(() => computeCanvasBounds(font, 1));
  }
  const textMetrics = cardBoundsCache[font]();

  const effectiveHeight = Math.round(
    textMetrics.actualBoundingBoxAscent +
      textMetrics.actualBoundingBoxDescent +
      2,
  );
  const effectiveWidth = Math.round(
    textMetrics.actualBoundingBoxRight +
      Math.min(textMetrics.actualBoundingBoxLeft, 0) +
      2,
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
