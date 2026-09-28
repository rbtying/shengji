import * as React from "react";

/** "faces": real card artwork. "simple": large rank and suit only, easier to read small. */
export type CardStyle = "faces" | "simple";

const KEY = "tta_card_style";
const EVENT = "tta-card-style";

export const getCardStyle = (): CardStyle => {
  try {
    return window.localStorage.getItem(KEY) === "simple" ? "simple" : "faces";
  } catch {
    return "faces";
  }
};

export const setCardStyle = (style: CardStyle): void => {
  try {
    window.localStorage.setItem(KEY, style);
  } catch {
    // The choice just won't be remembered.
  }
  window.dispatchEvent(new Event(EVENT));
};

export const useCardStyle = (): CardStyle => {
  const [style, setStyle] = React.useState<CardStyle>(getCardStyle);
  React.useEffect(() => {
    const update = (): void => setStyle(getCardStyle());
    window.addEventListener(EVENT, update);
    return () => window.removeEventListener(EVENT, update);
  }, []);
  return style;
};

/** "large": bigger cards for easier reading, with the hand in more rows. */
export type CardSize = "normal" | "large";

const SIZE_KEY = "tta_card_size";

export const getCardSize = (): CardSize => {
  try {
    return window.localStorage.getItem(SIZE_KEY) === "large"
      ? "large"
      : "normal";
  } catch {
    return "normal";
  }
};

export const setCardSize = (size: CardSize): void => {
  try {
    window.localStorage.setItem(SIZE_KEY, size);
  } catch {
    // The choice just won't be remembered.
  }
  window.dispatchEvent(new Event(EVENT));
};

export const useCardSize = (): CardSize => {
  const [size, setSize] = React.useState<CardSize>(getCardSize);
  React.useEffect(() => {
    const update = (): void => setSize(getCardSize());
    window.addEventListener(EVENT, update);
    return () => window.removeEventListener(EVENT, update);
  }, []);
  return size;
};
