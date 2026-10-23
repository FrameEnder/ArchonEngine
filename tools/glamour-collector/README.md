# Kawari Glamour Collector

A Chrome extension that collects Eorzea Collection glamours into a list for Kawari's party bots, exported as
`glamours.txt` (put it in the server's data folder, beside `config.yaml`).

1. `chrome://extensions` → turn on Developer mode → Load unpacked → pick this folder.
2. Open a glamour on ffxiv.eorzeacollection.com and press **Add to Kawari list** (bottom right).
3. Click the extension's toolbar icon. The gender and classes come from the page's "Fits" and "Intended for"; change
   the gender (ANY, Male, Female) or add classes from the dropdown (× removes one; none means any class), fix any
   lines if needed, then **Export glamours.txt**.

If the site changes and pages read wrong, press **Copy page HTML** on one and send the HTML over. The full description is in the fork's documentation, `docs/src/fork/tools.md`.
