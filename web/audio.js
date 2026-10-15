// Gets sound going on phones. Must load before mq_js_bundle.js.
//
// Browsers start an AudioContext suspended until the player touches the page, and
// macroquad tries to resume it on the first touchstart. iOS only lets a touchend (or a
// later event) do that, and macroquad gives up after that one attempt, so on an iPhone
// the game stayed silent. iOS also suspends the context again whenever the app goes
// to the background. So: keep track of every AudioContext the bundle creates and
// resume it on every tap or key press for as long as it isn't running.
(() => {
  // Play through the ring/silent switch like a video does. Web Audio on an iPhone is
  // otherwise muted by the switch; the game has its own music and sound toggles.
  if (navigator.audioSession) {
    navigator.audioSession.type = "playback";
  }

  const Base = window.AudioContext || window.webkitAudioContext;
  if (!Base) {
    return;
  }
  const contexts = [];
  window.AudioContext = class extends Base {
    constructor(...args) {
      super(...args);
      contexts.push(this);
    }
  };

  const wake = () => {
    for (const ctx of contexts) {
      if (ctx.state !== "running") {
        ctx.resume().catch(() => {});
        // Older iOS also wants a sound started inside the gesture itself.
        const src = ctx.createBufferSource();
        src.buffer = ctx.createBuffer(1, 1, 22050);
        src.connect(ctx.destination);
        src.start(0);
      }
    }
  };
  for (const type of ["touchend", "pointerup", "mousedown", "keydown"]) {
    window.addEventListener(type, wake, { capture: true, passive: true });
  }
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "visible") {
      wake();
    }
  });
})();
