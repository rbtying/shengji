import { State } from "./State";

export const localStorageState = <T>(
  key: string,
  extractor: (value: any) => T,
  serializer: (t: T) => any
): State<T> => {
  return {
    loadDefault: () => extractor(window.localStorage.getItem(key)),
    persist: (before: T, after: T) => {
      window.localStorage.setItem(key, serializer(after));
    },
  };
};

export const booleanLocalStorageState = (
  key: string,
  defaultValue = false
): State<boolean> =>
  localStorageState(
    key,
    (value: any): boolean => value === "on" || defaultValue,
    (state: boolean) => (state ? "on" : "off")
  );

export const stringLocalStorageState = (
  key: string,
  defaultValue = ""
): State<string> =>
  localStorageState(
    key,
    (value: any): string => (typeof value === "string" ? value : defaultValue),
    (state: string) => state
  );

export const numberLocalStorageState = (
  key: string,
  defaultValue = 0
): State<number> =>
  localStorageState(
    key,
    (value: any): number =>
      value != null && !isNaN(value) ? parseInt(value, 10) : defaultValue,
    (state: number) => state
  );

export function JSONLocalStorageState<T>(
  key: string,
  defaultValue: T
): State<T> {
  return localStorageState(
    key,
    (value: any): T => {
      try {
        const val = JSON.parse(value);
        if (val !== undefined && val !== null) {
          return val;
        } else {
          return defaultValue;
        }
      } catch (e) {
        return defaultValue;
      }
    },
    (state: T) => JSON.stringify(state)
  );
}
