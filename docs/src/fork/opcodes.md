# Fork opcodes

Opcodes change with every game patch. Upstream keeps theirs in `resources/data/opcodes.yml`; the fork keeps the ones
it found itself in **`resources/data/fork_opcodes.yml`**, so upstream's patch updates merge without conflicts. See also
upstream's [Opcodes](../opcodes.md) page.

## How it works

* **Build** (`core` build script): fork entries are merged into the same generated enums
  (`ClientZoneIpcType`, `ServerZoneIpcType`…). Each entry has `name`, `comment`, `size` (data only), `opcode`,
  `found_on` (game version), `how_to_find`, and optional `find` rules.
* An entry whose `found_on` isn't `SUPPORTED_GAME_VERSION` (`constants.yml`) or that has no opcode is **outstanding**:
  the build warns and gives it a placeholder opcode the game never uses, so it can't be mistaken for another packet.
* If upstream adds the same name, upstream's wins and the build says the fork entry can go.
* **Startup scan** (`servers/world/src/fork_opcodes.rs`, `exe_scan`): when `ffxivgame.ver` matches the supported
  version, the server reads `ffxiv_dx11.exe` from the game folder and runs each entry's finders:
  * `immediate`: search a byte `pattern` (`??` = any byte); the opcode is the `width`-byte number at `offset`. With
    `with: <name>`, only matches inside a function that also writes that packet's known opcode count.
  * `handler`: the pattern is in the client's handler for the packet; the opcode is the dispatcher case that leads
    there. Packets that share one handler (several sizes of one packet) can only be *confirmed* this way.
  * An outstanding opcode that's found is used immediately and the log says what to save in the YAML. A known one is
    checked, so a finder that stops working is noticed before the opcode is needed. A result that is already another
    packet's opcode is refused.
* **Remapping** (`packet::remap`): the enums keep the placeholder; incoming packets with the real opcode are read as
  the fork packet, outgoing ones are sent with the real opcode (also used for the scrambler keys).
* In game, `!forkopcodes` lists them; when an unknown client packet has an outstanding entry's size, the log says which
  it probably is.

Since upstream's 7.57h1 (2026.10.16, merged on the `upstream-7.57h1` branch, in `master` since 2026-10-21) every entry is found on 2026.10.16 (see
[Finding them on 7.57h1](#finding-them-on-757h1) below). 38 of the 47 have finders (2026-10-29; 35 of 39 on 2026-10-26, 14 before 2026-10-25), so the startup scan checks them on each start and re-finds them
after the next patch, and every one is in the shipped fingerprints. The four without can't have one: their code is
the same as other packets' (RequestPlayerRetainerInfo, a request without arguments; GatheringPointPhase; the two
ItemMarketBoards, which share a handler); after a patch only the fingerprints find them (the ItemMarketBoards as a
pair of candidates). Of the five added since, EventResume16 and LotteryDailyBoard can't have one either (`suggest-finders`
finds their code isn't unique); EventYieldText16, EventYieldText24 and CompanionName have none yet, though `suggest-finders`
writes one for each on 2026.10.16.

## Current entries (game 2026.10.16, patch 7.57h1)

| Name | Direction | Opcode | Earlier | Size | What it is |
|---|---|---|---|---|---|
| EventYieldText16 | client → server | 945 | | 16 | An event's text answer of up to 9 bytes: the handler, scene, yield ID, then the text from byte 7 (a chocobo's name; Bob's Chronofoil capture 0845b9a2). See [Grand Companies](grand-company.md#my-little-chocobo). |
| EventYieldText24 | client → server | 137 | | 24 | The same for 10 to 17 bytes (capture bab8c255). |
| CompanionName | server → client | 157 | | 40 | The chocobo's name as the server took it (0, then the name), right before the text answer's EventResume (both captures). |
| EventResume16 | server → client | 529 | | 72 | Resumes a scene with 9 to 16 parameters: Out on a Limb's and The Finer Miner's swings (Bob's Chronofoil capture deaf7e28). See [The Gold Saucer](gold-saucer.md#arcade-machines). |
| LotteryDailyBoard | server → client | 326 | | 136 | A Mini Cactpot ticket just bought: the day, the cell showing, the payouts (deaf7e28). See [The Gold Saucer](gold-saucer.md#the-mini-cactpot). |
| RideShootingScore | client → server | 796 | | 16 | Air Force One's score as the ride ends: 6, the score, 0-5, the silvers, bronzes and ruby bombs hit; sent again and again until leaving (Bob's Chronofoil captures deaf7e28 and 2c98f9e8). Finder. See [The Gold Saucer](gold-saucer.md#air-force-one). |
| RideShootingEnd | server → client | 318 | | 40 | Sent as Air Force One ends, before its result: the same 40 bytes both times, meaning unknown (the same captures). Finder. |
| EventReturnHandler16 | client → server | 555 | 856 (09.15), 200 (10.06) | 72 | A scene finished with up to 16 results, e.g. a three-item NPC trade returns 12 (Surveying the Damage). Finder. |
| EventNotice2 | server → client | 834 | 386 (10.06) | 16 | A quest/event notice with 0–2 arguments: the progress pop-up. Finder (confirms only, see below); 10.16's from a retail capture. |
| EventNotice4 | server → client | 467 | 210 (10.06) | 24 | The same with 3–4 arguments. Finder (confirms only); 10.16's from a retail capture. |
| SetAdventurerPlatePortrait | client → server | 431 | 107 (10.06) | 56 | The client sets its adventurer plate's portrait. Finder: the sender writing the opcode and the 72-byte size. |
| UpdateAdventurerPlate | client → server | 350 | 795 (10.06) | 200 | The client saves its adventurer plate (CharaCardData). Finder: the sender writing the opcode, then the 216-byte size. See [Adventurer Plate](character-windows.md#adventurer-plate). |
| MarketBoardOpened | client → server | 319 | 156 (09.15), 683 (10.06) | 8 | The marketboard window opened; answered with an empty search result. Seen in Bob's 2026-10-20 bundle as the window opened. Finder. |
| MarketBoardPurchaseHandler | client → server | 245 | 765 (09.15), 258 (10.06) | 40 | Buying a marketboard listing. Seen in Bob's 2026-10-20 bundle (his purchases); FFXIVOpcodes' 7.57h agrees. Finder. See [Marketboard](marketboard.md). |
| MarketBoardPurchase | server → client | 353 | 665 (09.15), 610 (10.06) | 16 | A purchase went through: item and quantity. Finder (written by `suggest-finders`; it reads 610 on 10.06). |
| RequestPlayerRetainerInfo | client → server | 348 | 580 (09.15), 818 (10.06) | 8 | Asks for the character's retainers as a retainer's sale list opens. See [Retainers](retainers.md). |
| RequestRetainerSaleHistory | client → server | 372 | 243 (09.15), 274 (10.06) | 8 | Asks for a retainer's sale history (their ID). Finder (written by `suggest-finders`; it reads 274 on 10.06). |
| SetRetainerAppearance | client → server | 463 | 670 (10.06) | 32 | The new retainer's voice and CustomizeData from the vocate's editor. Finder. |
| RetainerStateNotice | server → client | 987 | 897 (09.15), 144 (10.06) | 8 | Before every RetainerState and after a retainer's containers. Finder (written by `suggest-finders`; it reads 144 on 10.06). |
| RetainerState | server → client | 735 | 908 (09.15), 249 (10.06) | 56 | A retainer called (4), sent away (3) or hired (1). Finder (written by `suggest-finders`; it reads 249 on 10.06). |
| PlayerRetainerInfo | server → client | 715 | 599 (09.15), 712 (10.06) | 480 | The character's retainers for the marketboard. Finder (written by `suggest-finders`; it reads 712 on 10.06). |
| ItemMarketBoardInfo | server → client | 471 | 313 (09.15), 387 (10.06) | 32 | The asking price of an item a retainer has on the market. |
| ItemMarketBoardSummary | server → client | 290 | 442 (09.15), 303 (10.06) | 8 | How many items a retainer has on the market. |
| RetainerSaleHistory | server → client | 551 | 161 (09.15), 832 (10.06) | 1048 | A retainer's sale history. Finder (written by `suggest-finders`; it reads 832 on 10.06). |
| LootMessage | server → client | 154 | 351 (09.15), 190 (10.06) | 32 | Loot: kind 3 an item obtained directly (an enemy's drop), 9 gil, 0/1 need/greed rolls and winners. See [Combat](combat.md#defeating-enemies). Finder (written by `suggest-finders`; it reads 190 on 10.06). |
| LootRoll | client → server | 766 | | 16 | Need, Greed or Pass on a coffer's item: the coffer, the slot, 1/2/5. From Bob's Sastasha capture (2026-10-22). See [Coffers and Loot](loot.md). Finder (written by `suggest-finders`; it reads 231 on 10.06). |
| LootList | server → client | 272 | | 88 | An opened coffer's items and the time to roll. Same capture. Finder (written by `suggest-finders`; it reads 271 on 10.06). |
| LootItem | server → client | 975 | | 56 | One of a coffer's items: slot, unique, time left. Same capture. Finder (written by `suggest-finders`; it reads 334 on 10.06). |
| LootRollResult | server → client | 462 | | 16 | The answer to LootRoll: rolled (and the roll), passed or refused. Same capture. Finder (written by `suggest-finders`; it reads 285 on 10.06). |
| DutySupportContent | server → client | 701 | | 8 | The answer to ClientTrigger 2650 (a duty's entrance asking for its Duty Support window): the DawnContent, then 0. From Bob's capture of 2026-10-23 (Sastasha's entrance: 200, 0). See [Party NPCs](party-npcs.md#registering). Finder (written by `suggest-finders`; it reads 976 on 10.06). |
| GatheringPointPhases | server → client | 309 | 467 (06.18), 250 (09.01), 314 (09.15), 533 (10.06) | 160 | The gathering points' phase table, once at login: which position of each point the client shows. From Bob's captures (every login) and its handler in the 10.16 client. Finder: its handler. See [Gathering](gathering.md#where-the-points-are). |
| GatheringPointPhase | server → client | 845 | | 8 | One phase slot changes (slot, phase): a used-up point comes back at another position. From its handler in the 10.16 client and Bob's capture of 2026-10-24 (with ActorControlSelf 109 as a third point is used up). No finder (its handler's code is the same as two others'). See [Gathering](gathering.md#integrity-and-moving-on). |
| GatheringChainCount | server → client | 618 | | 8 | The gathering chain at a node (u32): after each gather from the second in a row, and 0 as the window closes. From Bob's capture of 2026-10-24. Finder: its handler. See [Gathering](gathering.md#at-the-node). |
| CraftingActionPreviews | server → client | 347 | | 424 | What each crafting action would do now (21 entries: action, efficiencies, progress, quality, state), before the Synthesis window opens and with every step. Same capture. Finder: its handler. See [Crafting](crafting.md#craftingactionpreviews). |
| RecordCraftingLog | server → client | 520 | | 8 | A recipe made for the first time: one byte of the completed recipes bitmask (u16 index, u8 new value). From Bob's capture of 2026-10-24 (recipe 3812: `[476, 0x08]`). Finder: its handler (2026-10-26). See [Crafting](crafting.md#the-crafting-log). |
| GlamourDresser | server → client | 436 | | 2408 | One page of the glamour dresser (page, 400 items, their stains), twice as it opens. From Bob's capture of 2026-10-24 (the Hourglass) and its dispatcher case in the 10.16 client (vtable entry 0x1348). Finder (written by `suggest-finders`; it reads 202 on 10.06). See [Glamour](glamour.md). |
| GlamourPlates | server → client | 400 | | 1440 | The 20 glamour plates. Same capture (vtable entry 0x1350). Finder (written by `suggest-finders`; it reads 649 on 10.06). |
| GlamourPlateUpdated | server → client | 621 | | 80 | One plate after it was saved. Same capture (vtable entry 0x1358). Finder (written by `suggest-finders`; it reads 868 on 10.06). |
| SaveGlamourPlate | client → server | 156 | | 248 | A glamour plate saved at the dresser. Same capture. Finder (written by `suggest-finders`; it reads 716 on 10.06). |
| EquipGearsetGlamour | client → server | 605 | | 128 | Equipping a gearset with a glamour plate linked: EquipGearset's layout with the plate's number (from 1) at byte 60, then 56 bytes. From Bob's Chronofoil captures of 2026-10-24. Finder (written by `suggest-finders`; it reads 265 on 10.06). See [Glamour](glamour.md#applying-a-plate). |
| ArmoireContents | server → client | 772 | | 136 | The armoire's bitmask (ClientTrigger 423). Same capture (vtable entry 0xEB8). Finder (written by `suggest-finders`; it reads 681 on 10.06). |
| ArmoireInfo | server → client | 445 | | 504 | Sent with it (ClientTrigger 427); 1 and zeros, meaning unknown. Same capture (vtable entry 0xEC0). Finder (written by `suggest-finders`; it reads 948 on 10.06). |
| EventReturnHandler128 | client → server | 866 | | 520 | A scene's return of up to 128 results (the craft's materials list). Same capture. Finder: the size it's sent with. See [Crafting](crafting.md#the-flow). |
| LootResult | server → client | 807 | | 16 | Who won a coffer's item, with what roll (0xE0000000 when everyone passed). Same capture. Finder (written by `suggest-finders`; it reads 521 on 10.06). |
| ActionRequestGroundTargeted | client → server | 492 | 891 (09.15), 846 (10.06) | 24 | A ground-targeted action (Asylum...): the action, its type, the sequence, the rotation, where on the ground. Not yet seen on Kawari; the layout is from Bob's captures. Handled as an action on oneself for now. Finder. |
| Achievements | server → client | 985 | 991 (10.06) | 552 | The completed achievements and the last five, the reply to client trigger 1001. Finder: its handler (HandleAchievementsPacket). See [Achievements](character-windows.md). |
| ChocoboTaxiStart | server → client | 538 | | 16 | A chocobo porter's ride begins: the ChocoboTaxi row, its Location, 8 bytes of 0 (Bob's Chronofoil captures cc62ad94, 1fac2416, 2528a32d: nine rides). See [Travel](travel.md#chocobo-porters). |
| ChocoboTaxiCheckpoint | client → server | 534 | | 8 | A point of the porter's route passed: the ChocoboTaxi row, the point's number (the same captures). |
| ChocoboTaxiArrived | client → server | 518 | | 8 | The route's end: the last point's number, then 0 (the same captures). |

The client picks the event return packet by result count (2, 4, 8, 16…); EventReturnHandler16's finder looks for its
16 case writing the opcode and the 88-byte size, and uses `with: EventReturnHandler2` because the yield sender has
identical code. All notice sizes share one handler, so their finder only confirms them; a recording said which size is
which. The `EventNotice` packet (`core`) was checked byte for byte against retail.

## Finding them on 7.57h1

The startup scan re-found the six with finders from Bob's 7.57h1 server log (2026-10-20 bundle); the rest were found
from Bob's 2026.10.06 and 2026.10.16 `ffxiv_dx11.exe` side by side, with `kawari-opcodescan` and capstone (scripts not
kept; the method is below):

* **FFXIVOpcodes' 7.57h table** (2026-10-20) has 39 of upstream's 40 shared 7.57h1 opcodes the same (FreeCompanyInfo
  differs), and its 7.57 table has every one of its names that the fork also has at the fork's 2026.10.06 value, so its
  names mean the same packets. It gives RetainerState, PlayerRetainerInfo, ItemMarketBoardInfo/Summary,
  RetainerSaleHistory, MarketBoardPurchase, MarketBoardPurchaseHandler and ActionRequestGroundTargeted; each was
  checked against the executables below.
* **Server packets**: each opcode's case in the zone dispatcher (the 525-case switch, table at `0x18DA592` on 10.06,
  `0x18DABF3` on 10.16), disassembled with the function it calls, has the same code on both versions. LootMessage's
  matches 154 and 821 (821 is upstream's UpdateClassInfo); RetainerStateNotice's only 987. Opcodes sharing a case
  (ItemMarketBoardInfo and Summary; the five notices) can't be told apart this way.
* **Client packets**: the function that sends each writes `mov [rsp+20h], opcode` and the size with its header; the
  matching site on 10.16 has the same code after it. Requests without arguments have the same code as each other, but
  the marketboard and retainer senders sit in the same order 0x640 bytes on (MarketBoardPurchaseHandler 0x9716C0 →
  0x971D00), which gives RequestPlayerRetainerInfo 348. RequestRetainerSaleHistory has a twin with near-identical code
  (810), so the closest match (372, identical) was taken.
* **The notices**: in 10.16's dispatcher the five notice cases (467, 561, 834, 927, 928) are one block that reads the
  handler ID, notice ID, argument count and the arguments from the packet (+0x10, +0x14, +0x15, +0x18 with the IPC
  header). The client doesn't use the opcode to know the size, so any two of the five show notices correctly; 467 and
  561 were taken at first. Bob's capture of the Limsa opening (2026-10-20) has retail's: the 16-byte notices (Close to
  Home's, the aetheryte's) are **834**, the 24-byte ones (quest 111's, three arguments) **467**.

Four new finders (MarketBoardOpened, MarketBoardPurchaseHandler, ActionRequestGroundTargeted, SetRetainerAppearance)
are each unique in both versions and read 10.06's opcode from 10.06's executable and 10.16's from 10.16's. The
others (the retainer requests, and the server packets other than the notices and Achievements) still have none.

## Other packet facts found by the fork

| Packet | Fact |
|---|---|
| Actor control 925 `ServerRequestReply` | Reply to client trigger 9003 (ServerRequestCallbackManager), e.g. the quest battle prompt, or command 4 at a summoning bell. |
| Client trigger 9003 (`OpenUnk1`) | Callback slot, command, then two parameters: at a summoning bell, command 2 asks for the retainer list and 3 for one retainer (the parameters are their ID). See [Retainers](retainers.md). |
| ItemOperation types | Retainers: BASE_INVENTORY_ACTION + 22 entrust, + 23 withdraw, + 24 gil, + 25 put up for sale (+ 26 from the crystals), the same offsets on 2026.09.15 (Move 0x203) and 2026.10.06 (Move 0x123). |
| Opcode 308 | Upstream's CreateLocalLinkshellRequest; on 2026.10.06 also an event's text answer (a retainer's name), 48 bytes. |
| EventResume with 8 results | Retail sent it as **329** on 2026.10.06 (the retainer vocate); upstream's EventResume8 is 513 (FFXIVOpcodes: ResultDialog). Not changed, needs checking. |
| DirectorVars | First byte (Kawari's `DirectorData.flag`) is the director's sequence. |
| DirectorPopupMessage (162) | The battle talk window, see [Quest battles](quest-battles.md#director-script). Its kind is one byte. |
| PortraitsInformation (182) | The adventurer plate's portrait (BannerData), the reply to client trigger 3200. |
| Actor controls 514, 515, 538, 600 | Achievement progress, completion, near-completion lists; see [Achievements](character-windows.md). |
| Actor controls 542, 543 | A Hunting Log target counted (note, target, count) and an entry finished (note, class); see [Hunting Log](hunting-log.md). |
| Client triggers 1000, 1001, 1002 | Achievement progress, the completed list, the near-completion lists. |
| QuestActiveList / UpdateQuest | Entry layout, see [Quests](quests.md#what-the-server-keeps-for-a-quest). |
| AoeEffect8/16/24/32 | Bob's captures (39,333 packets): the effects start at byte 42 as in ActionEffect (40-byte header, bytes 34–41 zero), 64 bytes a target, the target list at the next multiple of 8, then a packed position (0x7FFF ×3 for none) and 6 zero bytes: `align8(42 + 64N) + 8N + 12` = 636, 1212, 1788, 2364 bytes. Kawari's struct had them at 48 (fixed; it isn't sent yet: area actions show one target plus flying text). 2026.10.06's AoeEffect16/24/32 are 912/158/630 per FFXIVOpcodes. Retail also sends AoeEffect8 with no targets for area actions that hit nobody. |
| Opcode 206 | On 2026.10.06 it's FFXIVOpcodes' CFPreferredRole (a role byte per roulette, sent 113 times unprompted in the captures), not upstream's BeginStrategyBoardSession; the "board data" Kawari logged from it is that. Upstream's table, so not changed here. |
| Sizes upstream has wrong (every retail packet agrees) | EventScene255 is 1048 (no tail pad when 28 + 4N is a multiple of 8), DeepDungeonParty 24, StatusEffectListDouble 744 (30 statuses, then a whole StatusEffectList, its statuses at 380), TitleList 120, client CrossRealmListingsRequest2 64. In upstream's table, so only noted. |
| ActorControl 930 PlayDismountAnimation | Where the player lands: packed x, packed y, packed z with the quantized rotation in the top half (all 9 of Bob's within 0.03 yalms of his position). Kawari sent fixed numbers; it now sends the player's position (`dismount_landing`). |
| ActorControl 6 Defeated, 1540/1541 | Defeated: the killer twice, 2 s after a death; 1540 a heal over time tick, 1541 a damage over time tick (see [Combat](combat.md)). |
| ContentFinderCommencing | `4, 0,0,0, 1, 0,0,0`, the ContentFinderCondition as a u32 at byte 8, then the ready count (byte 18) and party size (19). Kawari had 4 where the duty goes. |
| ZoneInit 811, SpawnObject 238, ActorControlSelf/ActorControl/ActorControlTarget 516/607/856 | Confirmed with Kawari Probe on 2026.09.15 while finding 925. |

## Finding a new one

1. Client packets: do what `how_to_find` says on Kawari; the server warns about the unknown packet (and suggests the
   entry when the size matches). Save the opcode and set `found_on`.
2. Server packets: record the same action on retail with the Kawari Recorder (Quest packets tab keeps the opcode and
   first 48 bytes of packets addressed to quests/directors) on the same game version.
3. Try candidates live with `!rawpacket` / `!rawsweep`.
4. Write a finder so the next patch finds it automatically: add the opcode to `fork_opcodes.yml`, record fingerprints
   (below), then `kawari-opcodescan <game exe> suggest-finders resources/data/opcode_fingerprints.json [older exe]`
   writes it (next section). Run `cargo test -p kawari opcode_match` before committing: it fails while a fork opcode
   is missing from the fingerprints or fingerprinted with another number.

## Writing finders automatically

`core/src/finder_gen.rs`, `kawari-opcodescan <exe> suggest-finders <fingerprints.json> [other exe] [all]`. From an
executable whose opcodes are known (the fingerprints must be from that version), it writes a `find:` entry for each
fork opcode without one (every one with `all`) and prints it as YAML to paste:

* **Client packets** (`immediate`): the function that writes the opcode (from the fingerprint) is cut into pieces
  around the opcode's `mov`; addresses, jumps' and calls' targets and numbers over 0x1000 are masked, and the
  shortest run of bytes that is unique in `.text` (with at least 8 unmasked) and still holds the opcode is kept, with
  the opcode's offset and width. Functions up to 6,000 instructions (some build the packet late in a long function).
* **Server packets** (`handler`): the handler the dispatcher's case calls; a pattern in it, or, when the handler only
  passes the packet on (and looks like many others), in the functions it calls (two levels, 16 functions).
* Each one is checked to read the right opcode on this executable, with exactly one match. With an older (or newer)
  executable it also says what each reads there (`# reads N on <other>`), which is the check that it survives a patch:
  compare that with a capture or with fingerprint matching (`match`).

Packets whose code is the same as another packet's can't have a finder (`# no finder` with why): see the four in
[Current entries](#current-entries-game-20261016-patch-757h1). The 20 written on 2026-10-25 (from 10.16) each read on
10.06 what fingerprint matching (10.16 → 10.06) finds, exactly, and the loot and retainer ones what the 10.06 captures
show.

## After a game patch

What happens on patch day, end to end (the server needs nothing from anyone until step 4):

1. **Update the game, start the server.** The world server compares the install's version with
   `SUPPORTED_GAME_VERSION`:
   * newer: it finds every zone opcode from the new `ffxiv_dx11.exe` with the fingerprints (about three seconds;
     [below](#every-opcode-after-a-game-update)), writes `auto-opcodes/<version>.json` and the new version's
     fingerprints, and the patch server lets that version in;
   * then the fork's finders run on the same executable (35 of 39 fork opcodes), and each one that reads something
     other than the opcode in use is logged with the value to save; the fingerprints' answer and the finder's are two
     independent methods, so when they agree the opcode is right;
   * client packets the game now sends with **another size** are warned about (`ClientZoneIpcType::X: the game now
     sends it with [..] bytes, not N`) and listed under `size_changed` in `auto-opcodes/<version>.json`: their layout
     changed and their reading needs a look even though the opcode is found.
2. **Read the log and `auto-opcodes/<version>.json`.** Candidates (packets the client handles the same way) are
   assigned in order; a wrong one shows as that packet doing nothing (try the others with `!rawpacket`). Client
   packets not found are learnt by size the first time they arrive.
3. **Play.** Packets not found go out as `0xFFFE` (ignored by the game), so a feature is off rather than broken.
4. **When upstream releases their update,** merge it (their `opcodes.yml`, constants and scrambler data), then:
   * save the fork's opcodes in `fork_opcodes.yml` from `auto-opcodes/<version>.json` and the finders' log lines, and
     set `found_on`;
   * record new fingerprints: `kawari-opcodescan <game> fingerprint resources/data/opcode_fingerprints.json
     resources/data/opcodes.yml resources/data/fork_opcodes.yml <version>` (the server also records them into
     `auto-opcodes/` when it runs on the supported version);
   * re-check the finders: `kawari-opcodescan <game> suggest-finders resources/data/opcode_fingerprints.json <old exe>
     all` shows each one still reads its opcode, and writes one for any new fork opcode;
   * `cargo test -p kawari opcode_match` (the fingerprints cover every fork opcode with the same numbers).
5. Anything still not found stays switched off (or outstanding) until found by hand
   ([Finding a new one](#finding-a-new-one)), and the constants under **Not covered** below are checked by hand.

## Every opcode after a game update

`core/src/opcode_match.rs`, `servers/world/src/auto_opcodes.rs`. When the game in the game folder is newer than
`SUPPORTED_GAME_VERSION`, the world server reads its `ffxiv_dx11.exe` at startup and finds **every zone opcode,
upstream's and the fork's**, from that executable alone: no recording, no old executable, nothing looked up by hand.
It takes about three seconds.

**Fingerprints.** While the opcodes are known, `fingerprint` records how the client's code treats each packet into
`resources/data/opcode_fingerprints.json` (recorded on 2026.10.16 from Bob's executable; about 200 KB). Only hashes of
each instruction's shape are kept (what it does, its registers, numbers under 0x1000, no addresses), with where the
code was and the packets' names: nothing of the game itself. For each packet:

* **Server packets** (the client handles them): the client's zone dispatcher reads the opcode out of the packet
  (`movzx r, word [packet + 2]`) and branches on it. `walk` follows that code for each opcode with the opcode's value,
  keeping track of registers holding the opcode (or numbers made from it), the packet pointer and the compare flags:
  compares, subtractions, jump tables (including byte-indexed ones), `cdqe`, and calls that are passed the opcode or
  the packet. A branch the opcode doesn't decide is followed both ways (a limited number of times) and the path that
  decides most is kept; a call that decides nothing is stepped over. What's recorded: the code of the opcode's case
  (40 instructions, calls one level down), how many opcodes share it, the handler it calls (for the order, below),
  where the opcode's path ends further in (many cases pass the packet to a handler that branches on the opcode again,
  e.g. ItemMarketBoardInfo and Summary), and, for packets that still share an end, where one of ~120 *second
  dispatchers* (other functions that read a packet's opcode and compare it) sends them.
* **Client packets** (the client builds them): the function that writes the packet's opcode and, within ten
  instructions, its size with the IPC header (`mov [base+x], opcode`, `mov [base+x+8], size + 16`); every packet's size
  is in the opcode files. The whole function (300 instructions, the opcode's own write marked) is recorded.

**Matching**, in the new executable:

1. The dispatcher: the biggest switch over opcodes whose start looks most like the recorded one.
2. Exact: the same code, shared by the same number of opcodes, **unique both ways** (two places with the same code in
   the old version, or in the new one, aren't exact.)
3. By order: the game's functions stay in the same order from one version to the next, so the handlers matched so far
   predict where each other one moved. The candidate exactly where the order says wins, then the most alike (at least
   half the same instructions), within 0x200 bytes (0x40 for client functions). Three rounds.
4. Inside a case shared by several opcodes: ends with exactly the same code; then second dispatchers. How alike ends
   are isn't used here: the ends of one compare tree are a few bytes apart and run on into each other (that's what put
   10.06's CurrencyCrystalInfo on UpdateItem's opcode before.)
5. Client packets: exact functions (built for one packet only in the old version) vote 3, clearly the closest vote 1,
   then the order of functions.

What comes back for each packet is an opcode, a list of candidates the client's code doesn't tell apart, or nothing.

**Tested** on Bob's 2026.10.06 (`ffxiv_dx11.exe`, built 2026-10-05) and 2026.10.16 (`ffxiv_dx11_1.exe`, built
2026-10-15, the supported version) executables against both versions' known opcodes, in both directions. The
fingerprints were re-recorded on 10.16 on 2026-10-25 (225 server, 116 client packets, now every fork opcode); the
10.16 → 10.06 run with them gives the same numbers, and every fork opcode found there agrees with the finders:

| | Server packets (211) | Client packets (112) |
|---|---|---|
| 10.06 → 10.16 | 195 exact, 16 among candidates (the right one always in the list), **0 wrong** | 98 exact, 14 not found, **0 wrong** |
| 10.16 → 10.06 | 192 exact, 19 among candidates, **0 wrong** | 98 exact, 14 not found, **0 wrong** |

The candidates: the notices (EventNotice2/4, five opcodes the client handles identically), EventResume2/4/8 (eight),
InitResponse and PingSyncReply (21: they end in the dispatcher's default case, so they're handled before it), the
three inventory packets (UpdateItem, ContainerInfo, CurrencyCrystalInfo), the two InventoryTransactions, the two
ItemMarketBoards, Mogpendium, SearchFellowshipsResults, and in the 10.16 → 10.06 direction three Crossworld linkshell
packets (12 opcodes with the same handling). The client packets not found are built differently (not
`opcode` then `size` next to each other) or by code shared with another packet (EventYield/EventReturnHandler16).

**At startup** (`auto_opcodes::adapt_to_game`, before the fork finders):

* The newest fingerprints older than the game are used: the shipped ones, or `auto-opcodes/fingerprints-*.json`
  recorded since, so it chains across several updates.
* Found opcodes replace the whole `ServerZoneIpcType` and `ClientZoneIpcType` lists (`remap::set_table`). In a
  replaced list, an incoming opcode that isn't in it is read as unknown (`0xF000` + its opcode, never as the packet
  that had that number before; `remap::real_unknown` gives it back for the log), and a packet that wasn't found goes
  out as `0xFFFE`, which the game ignores. Chat and lobby opcodes didn't change between 10.06 and 10.16 and aren't
  touched.
* Candidates: packets sharing a list take different ones in order (logged as a warning, with the others to try with
  `!rawpacket`).
* Client packets not found are **learnt by size**: the first unknown packet whose size only one of them has is it from
  then on (logged).
* Written to `auto-opcodes/<version>.json` (opcodes, guesses, what wasn't found, and the install's expansion versions),
  and this version's fingerprints to `auto-opcodes/fingerprints-<version>.json` from the opcodes found exactly. The
  patch server accepts a game version with a result there (and expansions no newer than the install's) as current.
* When the game *is* `SUPPORTED_GAME_VERSION` and the shipped fingerprints are for another version, its fingerprints
  are recorded instead.
* **Size changes:** every client packet's size is read from the new executable (`opcode_match::client_sizes`, the
  `size + 16` written next to the opcode) and compared with `opcodes.yml`'s; differences are warned about and saved
  as `size_changed`. `kawari-opcodescan <new exe> match <fingerprints>` prints them too. (Server packets' sizes aren't
  in the client's code in the same way, so they aren't checked.)

**Not covered:** a patch also changes `BASE_INVENTORY_ACTION`, the mail result codes and the scrambler tables in
`constants.yml`; no pattern that finds them in the client was found (2026-10-25). Obfuscation is off by default (`enable_packet_obsfucation`), so the scrambler isn't needed to play;
item moves and mail need upstream's (or a hand-found) value. Content patches can also change bitmask sizes and packet
layouts, which no opcode table fixes.
