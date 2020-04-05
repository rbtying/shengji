const mapObject = <T, Value>(
  array: T[],
  mapper: (t: T) => [string, Value],
): {[key: string]: Value} => {
  const result: {[key: string]: Value} = {};
  array.forEach((t: T) => {
    const [key, value] = mapper(t);
    result[key] = value;
  });
  return result;
};

export default mapObject;
