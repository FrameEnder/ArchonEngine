// Talking to your ArchonEngine ControlPanel (Kawari Web before its 2.26.0): its address (on your LAN or tailnet, e.g.
// http://192.168.1.20:8080 or http://myserver.tail1234.ts.net:8080) and an administrator's token (its Settings →
// Character sync), kept in chrome.storage.local as `kawariWeb`. Chrome asks once to let the extension reach that address (the manifest's
// optional host permissions), which is also what lets it call the API from the popup without CORS.
//
// The API (kawari_web/glamours.py): GET /api/glamours/ping, GET /api/glamours/classes, GET and POST /api/glamours.

async function kawariWebSettings() {
  const { kawariWeb = { url: "", token: "" } } = await chrome.storage.local.get("kawariWeb");
  return kawariWeb;
}

// "http://host:port" from what was typed (a missing scheme is http, a trailing path is dropped.)
function kawariWebOrigin(typed) {
  let text = (typed || "").trim();
  if (!text) return "";
  if (!/^https?:\/\//i.test(text)) text = `http://${text}`;
  try {
    return new URL(text).origin;
  } catch (error) {
    return "";
  }
}

// Asks Chrome for the address (must run from a click.)
async function kawariWebAllow(origin) {
  return chrome.permissions.request({ origins: [`${origin}/*`] });
}

// Calls the API: returns its JSON, or throws with what went wrong.
async function kawariWebCall(path, options = {}) {
  const { url, token } = await kawariWebSettings();
  if (!url || !token) throw new Error("Set your ArchonEngine ControlPanel address and token first.");
  let response;
  try {
    response = await fetch(`${url}${path}`, {
      ...options,
      headers: { Authorization: `Bearer ${token}`, ...(options.headers || {}) },
    });
  } catch (error) {
    throw new Error(`Couldn't reach ${url} (is this computer on the same LAN or tailnet?)`);
  }
  let body = null;
  try {
    body = await response.json();
  } catch (error) {
    throw new Error(`${url} answered ${response.status}, not ArchonEngine ControlPanel's API (one older than 2.21.0?)`);
  }
  if (!response.ok || !body.ok) throw new Error(body.error || `${url} answered ${response.status}`);
  return body;
}

// Fetches the class list and keeps it (`classList`) for the dropdown.
async function kawariWebRefreshClasses() {
  const body = await kawariWebCall("/api/glamours/classes");
  const classList = { roles: body.roles, groups: body.groups, classes: body.classes };
  await chrome.storage.local.set({ classList });
  return classList;
}

if (typeof module !== "undefined") {
  module.exports = { kawariWebOrigin };
}
