// Tells the game when the browser releases its pointer lock, and how far the mouse
// moved while it was held (see src/pointer.rs).
miniquad_add_plugin({
  name: "stygian_pointer",
  version: 2,
  register_plugin(importObject) {
    let locked = false;
    let lost = false;
    // macroquad rounds `movementX` down to whole pixels, which loses small moves to
    // the right and makes small moves to the left bigger. Add it up ourselves.
    let dx = 0;
    document.addEventListener("pointerlockchange", () => {
      const now = document.pointerLockElement === canvas;
      if (locked && !now) {
        lost = true;
      }
      locked = now;
      dx = 0;
    });
    canvas.addEventListener("mousemove", (e) => {
      if (locked) {
        dx += e.movementX;
      }
    });
    importObject.env.stygian_pointer_lock_lost = () => {
      const was = lost;
      lost = false;
      return was ? 1 : 0;
    };
    importObject.env.stygian_pointer_take_dx = () => {
      const moved = dx;
      dx = 0;
      return moved;
    };
  },
});
