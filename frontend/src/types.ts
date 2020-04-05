export type ITrump =
  | {
      Standard: {suit: string; number: string};
      NoTrump: null;
    }
  | {
      Standard: null;
      NoTrump: {number: string};
    };
