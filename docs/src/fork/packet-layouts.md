# Packet Layouts from the Client

`kawari-opcodescan <exe> layouts …` maps the zone packets' layouts from the game executable alone. It compares them
with Kawari's structs and lists where the two disagree. It reads the client's machine code, finds where each known
packet is read (server packets) or written (client packets), and records every access. Its output stays outside
the repository; only the findings and the names derived from them are written here.

## Running it

```sh
CARGO_TARGET_DIR=… cargo build --release -p kawari-opcodescan
kawari-opcodescan <ffxiv_dx11.exe> layouts resources/data/opcodes.yml resources/data/fork_opcodes.yml \
    <game version, e.g. 2026.10.16.0000.0000> core/src <out folder> [<FFXIVClientStructs folder>]
```

It writes two files to the out folder, both outside git:

* `layouts.json`: each packet's fields and the functions they were read in, plus the findings;
* `layouts.md`: the same as tables, with the findings first.

It prints how many packets it mapped and with what confidence. A run takes a few seconds.

The opcode files must be right for that executable, since the tool finds packets by opcode. After a game update,
first get the new opcodes ([Every opcode after a game update](opcodes.md#every-opcode-after-a-game-update)), then
run `layouts` on the new executable. Diff its findings against the last run's: a layout change shows as new `size`,
`padding-read` or `unknown-used` lines for that packet.

The ClientStructs folder is optional: a local checkout of
[FFXIVClientStructs](https://github.com/aers/FFXIVClientStructs) (its `FFXIVClientStructs` folder). It's only used to
name things:

* handlers whose `MemberFunction` signature matches, e.g. `PacketDispatcher.HandleQueueUpdatePacket`;
* the fields of the packet structs it describes (`[FieldOffset]` in `QueueUpdatePacket`, `ActorCastPacket`, …).

ClientStructs is read at run time and nothing from it is stored.

## How it works

`tools/opcodescan/src/layouts.rs` does the analysis. It uses iced-x86 and the dispatcher and client-packet finders in
`core/src/opcode_match.rs` (`packet_sites`).

* **Server packets.** The tool walks the zone dispatcher (2026.10.16: RVA `0x18d9be0`) to each opcode's case. The
  dispatcher holds the packet in R8 and copies it to r12. Cases use RDI, which is either:
  * the packet itself, or
  * a stack copy made by `0x1da7ec0` for scrambled packets (the tool follows that copy).

  The data starts 0x10 bytes in, after the IPC header. From the case, the tool follows the packet pointer through up
  to four calls deep, with budgets per function and per packet. It records each memory access through the pointer
  or a copy of it, with this information:
  * offset and width;
  * signedness (`movsx` gives `i`), float (`f`), and bool (a byte compared with 0);
  * array strides and counts from loop bounds.

  `__chkstk`, utility functions (over 1000 callers, or ClientStructs names such as memcpy/memset/alloc/free) and
  arguments left over from an earlier call are not followed. A memcpy of the packet records a `copy` of that length.
* **Client packets.** Each client packet's builder writes its opcode to `[rsp+x]`, its size plus 16 to `[rsp+x+8]`
  and its data from `[rsp+x+0x20]`. The tool records every write into that frame range.
* **Confidence** (per packet):
  * **high**: read in the handler or one call in;
  * **medium**: only deeper;
  * **low**: the case is shared with other opcodes, so the reads may belong to any of them;
  * **none**: nothing read was found (handlers reached only through virtual calls, or that pass the packet on in a
    way the tool doesn't follow).
* **Kawari's side** (`tools/opcodescan/src/kawari_structs.rs`). The tool parses `core/src` with syn and lays out
  each `ServerZoneIpcData`/`ClientZoneIpcData` variant the way binrw would. It understands:
  * `pad_before`/`pad_after`, `pad_size_to`, `count`, `map` (the type it maps from);
  * `ignore`/`calc`/`temp`, enum reprs, and consts.

  Server variants are laid out as Kawari writes them, client variants as it reads them. Variants with a runtime
  length (a `Vec` with `count = field`) are laid out up to that field.
* **Findings** (`compare`). Only the first three kinds are marked `confident`:
  * `unknown-used`: the client reads or writes inside a field Kawari calls `unk…`;
  * `padding-read`: the client reads bytes Kawari pads;
  * `type`: for example, the client reads a float where Kawari has an integer;
  * `width`: the client reads fewer bytes than Kawari's field has (often just a flags byte);
  * `size`: the client reads past Kawari's size (often a handler shared with a bigger packet).

  Read a finding against the handler before changing a struct. The tool shows where a byte is used, not what it
  means.

## Executables

| File (Bob's uploads) | Version string in the executable | Game version |
|---|---|---|
| `ffxiv_dx11.exe` | `rev14101345_2026/10/06` | 2026.10.06 |
| `ffxiv_dx11_1.exe` (uploaded 2026-10-20) | `rev14155597_2026/10/16` | 2026.10.16, 7.57h1, the server's `SUPPORTED_GAME_VERSION` |

The results below are from 2026.10.16.

## Results (2026.10.16, 2026-10-25)

| | Mapped | High | Medium | Low (shared case) | Nothing found |
|---|---|---|---|---|---|
| Server packets (`opcodes.yml` + `fork_opcodes.yml`) | 225 | 154 | 9 | 28 | 34 |
| Client packets | 103 | 98 | 0 | 0 | 5 |

Kawari's structs were laid out for 208 of 226 server variants and 105 of 116 client variants, to the end of each.

There were 778 findings, 623 of them confident:

| Kind | Confident | Check |
|---|---|---|
| `unknown-used` | 598 | 108 |
| `padding-read` | 25 | 4 |
| `width` | 0 | 39 |
| `size` | 0 | 4 |

Most of the `unknown-used` findings are inside big `unk` arrays, which tells you which bytes matter: PlayerSetup 179,
ExamineCharacterInformation 75, OccultCrescentSetup 68, StatusEffectListDouble 65.

ClientStructs named:
* 16 packet structs: ZoneInit, SpawnPlayer, ActorSetPos, PrepareZoning, SpawnNpc, GearsetDrawData, QueueUpdate,
  ContentsFinderNotification, SpawnObject, InventoryItem, HousingPortal, ActorCast, SpawnTreasure, CharaCard,
  SpectatorList, Achievements;
* 26 handlers.

Spot checks agree field by field with ClientStructs' `ActorCastPacket`, `SpawnObjectPacket` and
`InventoryItemPacket`.

### Changed in Kawari (2026-10-25)

Both Duty Finder packets are in `core/src/ipc/zone/server/mod.rs`, and both stay 40 bytes.

* **ContentFinderUpdate** (QueueUpdatePacket; the handler is `PacketDispatcher.HandleQueueUpdatePacket`). The
  client reads these fields:

  | Offset | Read as | Field |
  |---|---|---|
  | 0x0 | u8 | state |
  | 0x1 | u8 | class/job |
  | 0x2 | u8 | `language_flags` (part of `unk1` before) |
  | 0x8 | 8 bytes | the duty settings |
  | 0x10 | u8 | `roulette_id` (part of `unk2` before) |
  | 0x13 | bool | `began_queue` (part of `unk2` before) |
  | 0x14 | 5 × u32 | `content_ids`, which Kawari had as 5 × u16 |

  `content_ids` is now `[u32; 5]`. With u16s, a second queued duty landed in the first one's upper half.

  This is safe: the bytes are identical when one duty is queued, since the other IDs were 0. The new fields are sent
  as 0, as the old unk bytes were.
* **ContentFinderFound** (ContentsFinderNotificationPacket). Byte 0 is now `queue_state` (Kawari already sent 3
  there inside `unk1`). The bytes sent are identical.

The world server's construction of both (`servers/world/src/zone_connection/mod.rs`) was updated to match. No other
struct changed: the remaining findings are leads (below) that need a recording or a closer look at the handler first.

### Leads (confident findings worth checking next)

Server packets:

| Packet | The client reads | Kawari has |
|---|---|---|
| AcceptQuest, FinishQuest | u16 at 0x4 | padding |
| Countdown | u16 at 0x14, u8 at 0x18, bool at 0x19, u8 at 0x1a | `unk`, then padding |
| FriendRemoved | u32 at 0x8, u8 at 0xc | padding, `unk1` |
| MarketBoardPurchase | u32 at 0x4, bool at 0xc | padding |
| ZoneInit | f32 at 0x1c | padding |
| AoeEffect8 | f32 at 0x278 | padding after the packed position (retail: zero in every Copperbell Mines AoeEffect8). Since 2026-10-25 the struct mirrors ActionEffect's header, then `effects` per target slot and `targets` (a test against retail's Medica layout) |
| UnkClassRelated | bool at 0x2, u8 at 0x3 | padding |
| ContentFinderCommencing | u8 at 0x0 and 0x2, u32 at 0x8 | `unk1: [u8; 24]` |
| Mount | u8 at 0x2 and 0x3, u32 at 0x4, 0x8 and 0xc | `unk1: [u8; 14]` |
| AdventurerPlate | ClientStructs calls `unk5` SomeState | `unk5` |
| SpectatorList | ClientStructs calls `unk1` Flag | `unk1` |
| Achievements | ClientStructs calls the unks Unk218 and Unk220 | `unk` |

Client packets: the client writes a constant 0 into what Kawari pads in SetClientLanguage (u32 at 0) and InitRequest
(u64 at 0). It writes real values into Kawari's padding in these packets:
* StartCountdown (bytes 6 and 7);
* SocialListRequest (0xc);
* CreateFellowship (0x3c);
* the CWLS requests;
* ViewCrossRealmListing.

These `size` findings are worth a check:
* StatusEffectList and StatusEffectListDouble share a handler.
* HouseList.
* FreeCompanyInfo (low confidence; the client reads to 0x120).
* EventReturnHandler2 and EventYieldHandler2 share a builder.

**HaterList and EnmityList.** The client reads the entries from 0x4, eight bytes each (actor ID as a u32, enmity as
a byte at +4). That's where Kawari puts them: its `pad_after = 4` is after the list, not before it. The earlier
known-issues entry saying they're one field off was wrong, and has been removed.

## Client packets of the wrong size (2026-10-30)

A client packet holds several segments, each with its own size. Kawari read each segment's IPC straight from the
stream with its layout, so a layout shorter or longer than what the client sent left the stream in the wrong place and
the next segment's header was garbage: "no variants matched … Unexpected value for enum" on `segment_type`, and the whole
packet was dropped. In Bob's bundle 20261030-202136 that was, by its timing, the click on the Waking Sands' door: its StartTalkEvent was
in the dropped packet, so the server never started the warp and the client waited in its event (`!finishevent` had
nothing to finish). Now each segment is read from its own bytes (`packet/encryption.rs`, `decrypt`): a layout reading
less leaves the rest unread, one reading more gets zeros, and either way the server warns with the opcode and both
sizes ("… came with N bytes but its layout reads M"), so the next one is easy to find.

The one known so far: **CrossRealmListingsRequest2** (878, the Party Finder's) comes with 64 bytes on 2026.10.16 (16
times in Bob's Chronofoil captures), where upstream's `opcodes.yml` and layout have 40. Upstream's entry is left as it
is (fork rule); it now only warns.

## Limits

* **Virtual calls are not followed.** Handlers that pass the packet through a vtable show as "nothing found"; the
  case address is still listed so they can be read by hand.
* **Shared cases are flagged, not split.** When several opcodes share a case, the reads cover all of them.
* **The tool can't name fields that ClientStructs doesn't describe.** Names in Kawari come from a person reading the
  handler.

## Several packets in one read (2026-11-10)

A connection's TCP reads don't line up with the game's packets: one read can hold two packets, or part of one.
Upstream's servers hand each read to `parse_packet`, which reads the first packet and drops the rest. The **lobby**
now keeps what it has received and parses every whole packet in it (`kawari::packet::take_packets`, by the header's
size at byte 24; a packet that hasn't all arrived waits for the next read). Bob's character list came up empty now and
then before the game closed (exit code 1, no crash dump: Dalamud's crash handler log had nothing for it); in the
failed login of his bundle 20261110-081839 (07:29) the lobby logged the client logging in and then nothing until it
timed out 49 s later, and never asked the world server for the characters, where the next login (07:32) did at once:
the client's ServiceLogin was lost. The world server's zone and chat connections still parse one packet a read
([Known Issues](known-issues.md#client-packets-of-the-wrong-size-2026-10-30)).
