import * as React from "react";

/** Screens this short or shorter (iPhone SE, small windows) get the compact table layout. */
const QUERY = "(max-height: 740px)";

export const useShortScreen = (): boolean => {
  const get = (): boolean =>
    typeof window !== "undefined" && window.matchMedia !== undefined
      ? window.matchMedia(QUERY).matches
      : false;
  const [short, setShort] = React.useState<boolean>(get);
  React.useEffect(() => {
    if (window.matchMedia === undefined) {
      return;
    }
    const mql = window.matchMedia(QUERY);
    const onChange = (): void => setShort(mql.matches);
    mql.addEventListener("change", onChange);
    onChange();
    return () => mql.removeEventListener("change", onChange);
  }, []);
  return short;
};
