// The browser's prompt() dialog for the browser build (see src/prompt.rs).
miniquad_add_plugin({
  name: "stygian_prompt",
  version: 1,
  register_plugin(importObject) {
    const enc = new TextEncoder();
    const dec = new TextDecoder();
    const str = (ptr, len) => dec.decode(new Uint8Array(wasm_memory.buffer, ptr, len));
    let answer = new Uint8Array();
    importObject.env.stygian_prompt = (q, qlen, def, dlen) => {
      const text = window.prompt(str(q, qlen), str(def, dlen));
      if (text == null) {
        return -1;
      }
      answer = enc.encode(text);
      return answer.length;
    };
    importObject.env.stygian_prompt_read = (buf, len) => {
      new Uint8Array(wasm_memory.buffer, buf, len).set(answer.subarray(0, len));
    };
  },
});
