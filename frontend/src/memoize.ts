const memoize = <T>(f: () => T): (() => T) => {
  type State = { called: false } | { called: true; result: T };
  let state: State = { called: false };
  return () => {
    if (state.called) {
      return state.result;
    } else {
      state = { called: true, result: f() };
      return state.result;
    }
  };
};

export default memoize;
