//! Noticing when the player takes the mouse back. While playing, the game holds on
//! to the pointer (see `App::grabs_pointer`). In the browser that's the Pointer Lock
//! API, and the browser lets go on Esc without passing the key on, so the browser
//! build asks `web/pointer.js`. Native builds hear about it as a focus change.
//!
//! While the pointer is held, the browser build also reads how far the mouse moved
//! from `web/pointer.js`: macroquad rounds those moves down to whole pixels.

#[cfg(not(target_arch = "wasm32"))]
pub fn lock_lost() -> bool {
    false
}

/// How far the mouse moved sideways since the last call, in window pixels, while the
/// pointer was held. `None` where macroquad's own `mouse_position` is good enough.
#[cfg(not(target_arch = "wasm32"))]
pub fn take_dx() -> Option<f32> {
    None
}

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    /// 1 if the browser released the pointer lock since the last call, else 0.
    fn stygian_pointer_lock_lost() -> i32;
    /// Sideways mouse movement while locked since the last call, unrounded.
    fn stygian_pointer_take_dx() -> f64;
}

/// Lets the JS plugin check that it matches this build (see miniquad's `init_plugins`).
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn stygian_pointer_crate_version() -> u32 {
    2
}

/// Whether the browser released the pointer lock since the last call.
#[cfg(target_arch = "wasm32")]
pub fn lock_lost() -> bool {
    unsafe { stygian_pointer_lock_lost() != 0 }
}

/// How far the mouse moved sideways since the last call, in window pixels, while the
/// pointer was held.
#[cfg(target_arch = "wasm32")]
pub fn take_dx() -> Option<f32> {
    Some(unsafe { stygian_pointer_take_dx() } as f32)
}
