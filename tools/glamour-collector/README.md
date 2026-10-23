# Kawari Glamour Collector

A Chrome extension that collects Eorzea Collection glamours into a list for Kawari's party bots, exported as
`glamours.txt` (put it in the server's data folder, beside `config.yaml`).

1. `chrome://extensions` → turn on Developer mode → Load unpacked → pick this folder.
2. Open a glamour on ffxiv.eorzeacollection.com and press **Add to Kawari list** (bottom right).
3. Click the extension's toolbar icon: set each entry's gender (ANY, Male, Female) and classes (empty for any), fix
   any lines the page didn't read right, then **Export glamours.txt**.

If a page reads wrong, press **Copy page HTML** on it and send the HTML over so the reader can be matched to the
site's markup. The full description is in the fork's documentation, `docs/src/fork/tools.md`.
