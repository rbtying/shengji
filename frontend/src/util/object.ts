const mapValues = <A, B>(
  object: { [key: string]: A },
  mapper: (a: A) => B
): { [key: string]: B } => {
  const result: { [key: string]: B } = {};
  Object.keys(object).forEach((k) => {
    result[k] = mapper(object[k]);
  });
  return result;
};

const filter = <T>(
  object: { [key: string]: T },
  predicate: (key: string, value: T) => boolean
): { [key: string]: T } => {
  const result: { [key: string]: T } = {};
  Object.keys(object).forEach((key) => {
    const value = object[key];
    if (predicate(key, value)) {
      result[key] = value;
    }
  });
  return result;
};

export default {
  mapValues,
  filter,
};
