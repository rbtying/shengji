import memoize from './memoize';
const getContext = memoize(() => new AudioContext());

const beep = (vol: number, freq: number, duration: number) => {
  const beepContext = getContext();
  const v = beepContext.createOscillator();
  const u = beepContext.createGain();
  v.connect(u);
  v.frequency.value = freq;
  v.type = 'square';
  u.connect(beepContext.destination);
  u.gain.value = vol * 0.01;
  v.start(beepContext.currentTime);
  v.stop(beepContext.currentTime + duration * 0.001);
};

export default beep;
