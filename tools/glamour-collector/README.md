# Kawari Glamour Collector

A Chrome extension that collects Eorzea Collection glamours into a list for Kawari's party bots and sends it to your
Kawari Web (2.21.0 or later), which writes the server's `glamours.txt`.

1. `chrome://extensions` → turn on Developer mode → Load unpacked → pick this folder.
2. Click the extension's toolbar icon and open **Kawari Web**: your Kawari Web's address as this browser reaches it
   (LAN, e.g. `http://192.168.1.20:8080`, or tailnet, e.g. `http://myserver.tail1234.ts.net:8080`) and your token
   from Kawari Web → Settings → Character sync (an administrator's). **Connect**, and let Chrome reach that address.
   In Kawari Web, Dashboard → Bot glamours → **Read the class list from the game** once, for the Class dropdown.
3. Open a glamour on ffxiv.eorzeacollection.com and press **Add to Kawari list** (bottom right).
4. In the popup the gender and classes come from the page's "Fits" and "Intended for"; change the gender (ANY, Male,
   Female) or add classes from the dropdown (× removes one; none means any class), fix any lines if needed, then
   **Send to Kawari Web** (press twice to confirm). Bots that join from then on wear them.

**Export glamours.txt** still saves the file, for a server without Kawari Web (it goes in the server's data folder,
beside `world.db`).

If the site changes and pages read wrong, press **Copy page HTML** on one and send the HTML over. The full description is in the fork's documentation, `docs/src/fork/tools.md`.
