//! Asking the player for a line of text outside the game. A phone shows no keyboard
//! for a canvas, so the browser build uses the browser's own `prompt()` dialog
//! (registered by `web/prompt.js`). Native builds have a keyboard and never ask.

#[cfg(not(target_arch = "wasm32"))]
pub fn ask(_question: &str, _default: &str) -> Option<String> {
    None
}

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    /// Shows the dialog. Returns the answer's byte length, or -1 if it was cancelled.
    fn stygian_prompt(q: *const u8, q_len: usize, def: *const u8, def_len: usize) -> i32;
    /// Copies the last answer into `buf`.
    fn stygian_prompt_read(buf: *mut u8, buf_len: usize);
}

/// Lets the JS plugin check that it matches this build (see miniquad's `init_plugins`).
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn stygian_prompt_crate_version() -> u32 {
    1
}

#[cfg(target_arch = "wasm32")]
pub fn ask(question: &str, default: &str) -> Option<String> {
    let len = unsafe {
        stygian_prompt(
            question.as_ptr(),
            question.len(),
            default.as_ptr(),
            default.len(),
        )
    };
    if len < 0 {
        return None;
    }
    let mut buf = vec![0u8; len as usize];
    unsafe { stygian_prompt_read(buf.as_mut_ptr(), buf.len()) };
    String::from_utf8(buf).ok()
}
