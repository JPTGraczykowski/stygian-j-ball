//! Noticing when the player takes the mouse back. While playing, the game holds on
//! to the pointer (see `App::grabs_pointer`). In the browser that's the Pointer Lock
//! API, and the browser lets go on Esc without passing the key on, so the browser
//! build asks `web/pointer.js`. Native builds hear about it as a focus change.

#[cfg(not(target_arch = "wasm32"))]
pub fn lock_lost() -> bool {
    false
}

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    /// 1 if the browser released the pointer lock since the last call, else 0.
    fn stygian_pointer_lock_lost() -> i32;
}

/// Lets the JS plugin check that it matches this build (see miniquad's `init_plugins`).
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn stygian_pointer_crate_version() -> u32 {
    1
}

/// Whether the browser released the pointer lock since the last call.
#[cfg(target_arch = "wasm32")]
pub fn lock_lost() -> bool {
    unsafe { stygian_pointer_lock_lost() != 0 }
}
