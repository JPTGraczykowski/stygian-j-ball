// Offline play. Every file comes from the network when it can, so a new deploy shows
// up on the next visit; the copy cached last time is the fallback when offline.
const CACHE = "stygian-j-ball";
const FILES = [
  "./",
  "index.html",
  "mq_js_bundle.js",
  "storage.js",
  "prompt.js",
  "stygian-j-ball.wasm",
  "manifest.webmanifest",
  "icon-192.png",
  "icon-512.png",
];

self.addEventListener("install", (e) => {
  self.skipWaiting();
  e.waitUntil(caches.open(CACHE).then((c) => c.addAll(FILES)));
});

self.addEventListener("activate", (e) => e.waitUntil(self.clients.claim()));

self.addEventListener("fetch", (e) => {
  if (e.request.method !== "GET") {
    return;
  }
  e.respondWith(
    fetch(e.request)
      .then((res) => {
        if (res.ok) {
          const copy = res.clone();
          e.waitUntil(caches.open(CACHE).then((c) => c.put(e.request, copy)));
        }
        return res;
      })
      .catch(() =>
        caches.match(e.request, { ignoreSearch: true }).then((hit) => hit ?? Response.error()),
      ),
  );
});
