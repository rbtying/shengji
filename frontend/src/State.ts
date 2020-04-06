import mapValues from './util/mapValues';

export type State<T> = {
  loadDefault: () => T;
  persist: (before: T, after: T) => void;
};

export const noPersistence = <T>(loadDefault: () => T): State<T> => {
  return {
    loadDefault,
    persist: () => {},
  };
};

export const combineState = <S extends {}>(
  object: {[K in keyof S]: State<S[K]>},
): State<S> => {
  return {
    loadDefault: (): any =>
      mapValues(object, (p: State<any>): any => p.loadDefault()),
    persist: (before: any, after: any) => {
      Object.keys(after).forEach((k: string): any => {
        if (before[k] !== after[k]) {
          object[k as keyof S].persist(before[k], after[k]);
        }
      });
    },
  };
};
