const sum = (array: number[]): number => array.reduce((a, b) => a + b, 0);

type Equality<T> = (left: T, right: T) => boolean;
const identity = <T>(l: T, r: T): boolean => l === r;

// Subtracts one array from another. Works with arrays with duplicate values,
// and throws an exception if the smaller array is not completely contained in
// the larger one.
const minus = <T>(
  large: T[],
  small: T[],
  equality: Equality<T> = identity
): T[] => {
  const result = [...large];
  small.forEach((valueToRemove: T) => {
    const index = result.findIndex((t: T) => equality(t, valueToRemove));
    if (index >= 0) {
      result.splice(index, 1);
    }
  });

  return result;
};

const mapObject = <T, Value>(
  array: T[],
  mapper: (t: T) => [string, Value]
): { [key: string]: Value } => {
  const result: { [key: string]: Value } = {};
  array.forEach((t: T) => {
    const [key, value] = mapper(t);
    result[key] = value;
  });
  return result;
};

const range = <T>(count: number, fn: (idx: number) => T): T[] =>
  Array(count)
    .fill(undefined)
    .map((_, idx) => fn(idx));

const shuffled = <T>(array: T[]): T[] =>
  array
    .map((a) => ({ sort: Math.random(), value: a }))
    .sort((a, b) => a.sort - b.sort)
    .map((a) => a.value);

export default {
  mapObject,
  minus,
  range,
  sum,
  shuffled,
};
