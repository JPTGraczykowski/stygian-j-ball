//! Small text-file persistence. Native builds use the filesystem; the browser build
//! keeps each "file" in `localStorage`, keyed by its path, through the JS functions
//! that `web/storage.js` registers with miniquad.

use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
pub fn read(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn write(path: &Path, text: &str) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, text);
}

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    /// Byte length of the stored value, or -1 if there is none.
    fn stygian_storage_len(key: *const u8, key_len: usize) -> i32;
    fn stygian_storage_read(key: *const u8, key_len: usize, buf: *mut u8, buf_len: usize);
    fn stygian_storage_write(key: *const u8, key_len: usize, val: *const u8, val_len: usize);
}

/// Lets the JS plugin check that it matches this build (see miniquad's `init_plugins`).
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn stygian_storage_crate_version() -> u32 {
    1
}

#[cfg(target_arch = "wasm32")]
pub fn read(path: &Path) -> Option<String> {
    let key = path.to_string_lossy();
    let len = unsafe { stygian_storage_len(key.as_ptr(), key.len()) };
    if len < 0 {
        return None;
    }
    let mut buf = vec![0u8; len as usize];
    unsafe { stygian_storage_read(key.as_ptr(), key.len(), buf.as_mut_ptr(), buf.len()) };
    String::from_utf8(buf).ok()
}

#[cfg(target_arch = "wasm32")]
pub fn write(path: &Path, text: &str) {
    let key = path.to_string_lossy();
    unsafe { stygian_storage_write(key.as_ptr(), key.len(), text.as_ptr(), text.len()) };
}
