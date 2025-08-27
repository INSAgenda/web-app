self.addEventListener('install', function(event) {
    self.skipWaiting();
    console.log("Service worker installed");
});
  
self.addEventListener('fetch', function(event) {
    let request = event.request;

    if (request.url.includes("/api/") || request.url.includes("zebi") || request.url.includes("cas-login")) {
        return;
    }

    // All these paths are handled by the same app, and we serve the same index.html file on them
    let url = new URL(request.url);
    if (request.destination == "document" && (request. url.pathname == "/settings" || url.pathname == "/settings.html" || url.pathname == "/settings/"
        || url.pathname == "/agenda" || url.pathname == "/agenda.html" || url.pathname == "/agenda/"
        || url.pathname == "/mastodon" || url.pathname == "/mastodon.html" || url.pathname == "/mastodon/"
        || url.pathname == "/friends" || url.pathname == "/friends.html" || url.pathname == "/friends/"
        || url.pathname == "/stotra" || url.pathname == "/stotra.html" || url.pathname == "/stotra/"
        || url.pathname.startsWith("/survey/")
        || url.pathname.startsWith("/friend-agenda/")
        || url.pathname.startsWith("/event/"))) {

        request = new Request("/agenda", {
            body: request.body,
            cache: request.cache,
            destination: request.destination,
            headers: request.headers,
            method: request.method,
            priority: request.priority,
            redirect: request.redirect,
            url: new URL("/agenda", url.origin),
        });
    }

    event.respondWith(caches.match(request).then(function(response) {
        if (response !== undefined) {
            return response;
        } else {
            return fetch(request);
        }
    }));

    event.waitUntil(update(request));
});

function update(request) {
    caches.open("v1").then(function (cache) {
        fetch(request).then(function (response) {
            cache.put(request, response);
        });
    });
}
