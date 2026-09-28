import * as React from "react";

/** Screens at or below this width (CSS px) get the compact phone layout. */
export const NARROW_QUERY = "(max-width: 600px)";

const matches = (): boolean =>
  typeof window !== "undefined" && window.matchMedia !== undefined
    ? window.matchMedia(NARROW_QUERY).matches
    : false;

/** True on phone-sized screens; updates on rotation and window resizes. */
export const useNarrowScreen = (): boolean => {
  const [narrow, setNarrow] = React.useState<boolean>(matches);
  React.useEffect(() => {
    if (window.matchMedia === undefined) {
      return;
    }
    const mql = window.matchMedia(NARROW_QUERY);
    const onChange = (): void => setNarrow(mql.matches);
    mql.addEventListener("change", onChange);
    onChange();
    return () => mql.removeEventListener("change", onChange);
  }, []);
  return narrow;
};
