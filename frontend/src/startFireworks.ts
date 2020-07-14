import Confetti from "canvas-confetti";
var confetti: any = null;

export const startFireworks = (timeInSecond: number): void => {
  if (confetti == null) {
    confetti = Confetti.create(null, { resize: true });
  }

  var duration = timeInSecond * 1000;
  var animationEnd = Date.now() + duration;
  var defaults = { startVelocity: 30, spread: 360, ticks: 60, zIndex: 0 };
  var handler: any = null;

  function randomInRange(min: number, max: number): number {
    return Math.random() * (max - min) + min;
  }

  var interval: number;

  interval = setInterval(function () {
    var timeLeft = animationEnd - Date.now();

    if (timeLeft <= 0) {
      handler != null && window.removeEventListener("click", handler);
      return clearInterval(interval);
    }

    var particleCount = 500 * (timeLeft / duration);
    // since particles fall down, start a bit higher than random
    confetti(
      Object.assign({}, defaults, {
        particleCount,
        origin: { x: randomInRange(0.1, 0.3), y: Math.random() - 0.2 },
      })
    );
    confetti(
      Object.assign({}, defaults, {
        particleCount,
        origin: { x: randomInRange(0.7, 0.9), y: Math.random() - 0.2 },
      })
    );
  }, 250);

  setTimeout(() => {
    handler = window.addEventListener("click", (event) => {
      handler != null && window.removeEventListener("click", handler);
      clearInterval(interval);
    });
  }, 1000);
};
