// On an Eorzea Collection glamour page: a small panel to add the glamour to the list (kept in chrome.storage.local,
// shown and exported by the toolbar popup), and to copy the gear section's HTML when a page reads wrong.

(function () {
  if (document.getElementById("kawari-glamour-panel")) return;

  const panel = document.createElement("div");
  panel.id = "kawari-glamour-panel";
  panel.innerHTML = `
    <button type="button" data-kawari="add">Add to Kawari list</button>
    <button type="button" data-kawari="html" title="Copies the gear section's HTML, to send when a page reads wrong">Copy page HTML</button>
    <div data-kawari="status"></div>`;
  document.body.appendChild(panel);

  const status = panel.querySelector('[data-kawari="status"]');
  const say = (text) => {
    status.textContent = text;
    clearTimeout(say.timer);
    say.timer = setTimeout(() => (status.textContent = ""), 4000);
  };

  const pageUrl = () => location.href.split("#")[0];

  async function refresh() {
    const { entries = [] } = await chrome.storage.local.get("entries");
    const added = entries.some((entry) => entry.url === pageUrl());
    panel.querySelector('[data-kawari="add"]').textContent = added ? "In Kawari list ✓ (add again)" : "Add to Kawari list";
  }

  panel.querySelector('[data-kawari="add"]').addEventListener("click", async () => {
    const entry = kawariParseGlamour(document);
    if (entry.pieces.length === 0) {
      say("No gear found on this page: added with no pieces, fill them in from the popup.");
    }
    const { entries = [] } = await chrome.storage.local.get("entries");
    const existing = entries.findIndex((other) => other.url === entry.url);
    if (existing >= 0) {
      // Keep the gender chosen before, read the rest again.
      entry.id = entries[existing].id;
      entry.gender = entries[existing].gender;
      entries[existing] = entry;
    } else {
      entries.push(entry);
    }
    await chrome.storage.local.set({ entries });
    if (entry.pieces.length > 0) say(`${existing >= 0 ? "Updated" : "Added"}: ${entry.title} (${entry.pieces.length} pieces)`);
    refresh();
  });

  panel.querySelector('[data-kawari="html"]').addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(kawariPageHtml(document));
      say("Gear HTML copied.");
    } catch (error) {
      say(`Couldn't copy: ${error.message}`);
    }
  });

  chrome.storage.onChanged.addListener(refresh);
  refresh();
})();
