# Settings Backup (Save Data Bank)

The game's **System → Server Backup** (upload and download of *Client Settings* and *Character Configuration Data*)
keeps the files a character's settings live in on the server: gear sets, hotbars, macros, keybinds, the HUD (the
`FFXIV_CHR<content ID>` files, see [Auracite Backups](auracite.md#gear-sets-hotbars-macros-and-the-ui)). Kawari
doesn't serve it yet. This page is what's known, from Bob's captures of 2026-10-24/25 and the 7.57 client
(2026.10.16), and how to record what's missing.

## It isn't in the Chronofoil captures

Bob recorded one download and one upload of both kinds (captures `3326e01b`, 2 min, and `e1b6ee1b`, 3.5 min; which
one is which can't be told from them). Chronofoil records the lobby, zone and chat connections, and both captures
hold only what any login does:

* **Lobby:** the usual ServiceLogin, GameLogin, LoginEx and their replies (LoginReply, ServiceLoginReply,
  DistWorldInfo, XiCharacterInfo, DistRetainerInfo, GameLoginReply), plus keep-alives (segment types 7 and 8) and the
  encryption handshake (9 and 10). No other lobby opcode.
* **Zone and chat:** a normal login and a minute or two in town; no packet of a size or kind that could carry a file,
  no URL or token handed to the client (the only `https://` strings are in the welcome ServerNoticeMessages).

## Where it goes instead

The client opens a **separate TCP connection** for it, which Chronofoil doesn't record:

* Its host and port are two of the client's network settings, **`SaveDataBankHost`** and **`SaveDataBankPort`**, with
  `SaveDataBankMode` beside them. Their defaults, in the client's config table (7.57 `0x1400be22e`-`0x1400be2c3`), are
  `config-dl.ffxiv.com` and **55000** (0xD6D2). FFXIVClientStructs has them on NetworkModule (`SaveDataBankPort`
  +0x620, `SaveDataBankHost` +0x628, `SaveDataBankMode` +0x690), and the client copies them into each lobby
  connection's setup (7.57 `0x141cfb5c5`).
* Both can be set as launch arguments (`SaveDataBankHost=…`, `SaveDataBankPort=…`) like the lobby hosts, and
  Kawari's launcher auto-configuration already has empty `savedata_bank_server` / `savedata_bank_port` fields for it
  (`resources/web/templates/autoconfig.json`); upstream's hosts list has `config-dl.ffxiv.localhost`. With nothing
  listening there, the backup window fails as it would offline.
* The client keeps local copies under its user folder: `backup/FFXIV_BKSYS_NN.dat` (client settings) and
  `backup/FFXIV_BKCHR<content ID>_NN.dat` (a character's), and builds them from the same file list as the
  `FFXIV_CHR` folder (MACROSYS, COMMON, CONTROL0/1, MACRO, ADDON, UISAVE, GEARSET, KEYBIND, HOTBAR, LOGFLTR, GS, ACQ,
  ITEMFDR, ITEMODR; the table at 7.57 `0x1428b4b80`, read by the user-file code around `0x140143b70` and
  `0x140166650`-`0x14016bbb0`). There's also `cfgcopy/FFXIV_CFGCPY…` (copying settings between characters) and
  `configlist/` (HUD layouts), and the HUD sharing service `companion-config-share.finalfantasyxiv.com` (HTTPS,
  `/config/auth`, `/config/upload`, `/config/check`, `/config/download`), which is a different feature.

## Recording it

`tools/sdbrelay/sdbrelay.py` is a plain TCP relay that writes every byte both ways to a gzipped JSON-lines file (and
recognises FFXIV packet headers). On retail:

1. Run `python3 tools/sdbrelay/sdbrelay.py` (it listens on 127.0.0.1:55000 and relays to config-dl.ffxiv.com:55000).
2. Start the game with the launch arguments `SaveDataBankHost=127.0.0.1 SaveDataBankPort=55000` (XIVLauncher's
   *Additional launch arguments*, or the launcher's save data bank settings).
3. Upload and download both kinds of backup, then stop the relay. The file holds your own settings; keep it out of
   git and send it like a diagnostics bundle.

If the traffic turns out to be TLS-wrapped with the server checked by name, the relay sees only ciphertext and the
client may refuse it; that would show at once (the backup failing through the relay), and then the protocol has to
come from the client's code instead.

## What would follow

With a recording: a `servers/savedata` (or a part of the lobby server) answering on the port the launcher hands out,
storing each account's client settings and each character's files, and ArchonEngine ControlPanel importing an Auracite
(Kawari) backup's `config_files` into that storage, so a character's gear sets and hotbars would follow it to Kawari
without the plugin's restore.
