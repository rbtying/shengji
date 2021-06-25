import * as React from "react";
import Timeout from "./Timeout";
import * as confetti from "canvas-confetti";

interface IProps {
  confetti: string;
  clearConfetti: () => void;
}

const Confetti = (props: IProps): JSX.Element => {
  const duration = 30000;
  const canvasRef = React.useCallback((canvas) => {
    if (canvas !== null) {
      const c = confetti.create(canvas, {
        resize: true,
      });
      const animationEnd = Date.now() + duration;
      const defaults = { startVelocity: 30, spread: 360, ticks: 60, zIndex: 4 };
      const randomInRange = (min: number, max: number): number => {
        return Math.random() * (max - min) + min;
      };

      const interval = setInterval(() => {
        const timeLeft = animationEnd - Date.now();
        if (timeLeft < 0) {
          // Don't clear the interval here; it'll get cleared when the
          // component is unmounted.
          return;
        }
        const percentDone = timeLeft / duration;
        const particleCount = 200 * percentDone;
        // since the particles fall down, start a bit higher than random
        const number = Math.random() * 4 * percentDone * percentDone;
        for (let i = 1; i < number; i++) {
          c({
            ...defaults,
            particleCount: particleCount,
            origin: { x: randomInRange(0.2, 0.8), y: Math.random() - 0.2 },
            angle: randomInRange(65, 115),
          }).then(
            () => {},
            () => {}
          );
        }
      }, 500);

      return () => {
        clearInterval(interval);
        c.reset();
      };
    }
  }, []);
  return (
    <div
      style={{
        position: "fixed",
        height: "100%",
        width: "100%",
        background: "rgba(255, 255, 255, 0.8)",
        zIndex: 2,
      }}
      onClick={() => props.clearConfetti()}
    >
      <Timeout timeout={duration} callback={() => props.clearConfetti()} />
      <canvas
        ref={canvasRef}
        width={window.innerWidth}
        height={window.innerHeight}
        style={{ height: "100%", width: "100%" }}
      />
      <h1
        style={{
          position: "fixed",
          textAlign: "center",
          zIndex: 3,
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -50%)",
        }}
      >
        {props.confetti}
      </h1>
    </div>
  );
};

export default Confetti;
