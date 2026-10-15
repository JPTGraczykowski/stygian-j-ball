// Tells the game when the browser releases its pointer lock (see src/pointer.rs).
miniquad_add_plugin({
  name: "stygian_pointer",
  version: 1,
  register_plugin(importObject) {
    let locked = false;
    let lost = false;
    document.addEventListener("pointerlockchange", () => {
      const now = document.pointerLockElement === canvas;
      if (locked && !now) {
        lost = true;
      }
      locked = now;
    });
    importObject.env.stygian_pointer_lock_lost = () => {
      const was = lost;
      lost = false;
      return was ? 1 : 0;
    };
  },
});
