import {State} from './State';

export const localStorageState = <T>(
  key: string,
  extractor: (value: any) => T,
  serializer: (t: T) => any,
): State<T> => {
  return {
    loadDefault: () => extractor(window.localStorage.getItem(key)),
    persist: (t: T) => window.localStorage.setItem(key, serializer(t)),
  };
};

export const booleanLocalStorageState = (
  key: string,
  defaultValue = false,
): State<boolean> =>
  localStorageState(
    key,
    (value: any): boolean => value === 'on' || defaultValue,
    (state: boolean) => (state ? 'on' : 'off'),
  );

export const stringLocalStorageState = (
  key: string,
  defaultValue = '',
): State<string> =>
  localStorageState(
    key,
    (value: any): string => (typeof value === 'string' ? value : defaultValue),
    (state: string) => state,
  );
