import memoize from "./memoize";

const getContext = memoize(() => new window.AudioContext());

const beep = (vol: number, freq: number, duration: number): void => {
  if (window.AudioContext !== undefined) {
    const beepContext = getContext();
    const v = beepContext.createOscillator();
    const u = beepContext.createGain();
    v.connect(u);
    v.frequency.value = freq;
    v.type = "square";
    u.connect(beepContext.destination);
    u.gain.value = vol * 0.01;
    v.start(beepContext.currentTime);
    v.stop(beepContext.currentTime + duration * 0.001);
  } else {
    alert("Your browser doesn't support the beep feature! Beep!");
  }
};

export default beep;
