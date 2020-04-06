const mapValues = <A, B>(
  object: {[key: string]: A},
  mapper: (a: A) => B,
): {[key: string]: B} => {
  const result: {[key: string]: B} = {};
  Object.keys(object).forEach((k) => {
    result[k] = mapper(object[k]);
  });
  return result;
};

export default mapValues;
