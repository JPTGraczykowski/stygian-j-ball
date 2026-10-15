// localStorage-backed text files for the browser build (see src/storage.rs).
miniquad_add_plugin({
  name: "stygian_storage",
  version: 1,
  register_plugin(importObject) {
    const enc = new TextEncoder();
    const dec = new TextDecoder();
    const str = (ptr, len) => dec.decode(new Uint8Array(wasm_memory.buffer, ptr, len));
    const get = (key) => {
      try {
        return localStorage.getItem(key);
      } catch (e) {
        return null;
      }
    };
    importObject.env.stygian_storage_len = (k, klen) => {
      const v = get(str(k, klen));
      return v == null ? -1 : enc.encode(v).length;
    };
    importObject.env.stygian_storage_read = (k, klen, buf, len) => {
      const bytes = enc.encode(get(str(k, klen)) ?? "");
      new Uint8Array(wasm_memory.buffer, buf, len).set(bytes.subarray(0, len));
    };
    importObject.env.stygian_storage_write = (k, klen, v, vlen) => {
      try {
        localStorage.setItem(str(k, klen), str(v, vlen));
      } catch (e) {
        // Storage disabled or full: scores just won't persist.
      }
    };
  },
});
