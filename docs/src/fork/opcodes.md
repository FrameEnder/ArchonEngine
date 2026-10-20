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

Since upstream's 7.57h1 (2026.10.16, merged on `upstream-7.57h1`) every entry is found on 2026.10.16 (see
[Finding them on 7.57h1](#finding-them-on-757h1) below). Ten have finders, so the startup scan checks them on each
start and re-finds them after the next patch.

## Current entries (game 2026.10.16, patch 7.57h1)

| Name | Direction | Opcode | Earlier | Size | What it is |
|---|---|---|---|---|---|
| EventReturnHandler16 | client → server | 555 | 856 (09.15), 200 (10.06) | 72 | A scene finished with up to 16 results, e.g. a three-item NPC trade returns 12 (Surveying the Damage). Finder. |
| EventNotice2 | server → client | 467 | 386 (10.06) | 16 | A quest/event notice with 0–2 arguments: the progress pop-up. Finder (confirms only, see below). |
| EventNotice4 | server → client | 561 | 210 (10.06) | 24 | The same with 3–4 arguments. Finder (confirms only). |
| SetAdventurerPlatePortrait | client → server | 431 | 107 (10.06) | 56 | The client sets its adventurer plate's portrait. Finder: the sender writing the opcode and the 72-byte size. |
| UpdateAdventurerPlate | client → server | 350 | 795 (10.06) | 200 | The client saves its adventurer plate (CharaCardData). Finder: the sender writing the opcode, then the 216-byte size. See [Adventurer Plate](character-windows.md#adventurer-plate). |
| MarketBoardOpened | client → server | 319 | 156 (09.15), 683 (10.06) | 8 | The marketboard window opened; answered with an empty search result. Seen in Bob's 2026-10-20 bundle as the window opened. Finder. |
| MarketBoardPurchaseHandler | client → server | 245 | 765 (09.15), 258 (10.06) | 40 | Buying a marketboard listing. Seen in Bob's 2026-10-20 bundle (his purchases); FFXIVOpcodes' 7.57h agrees. Finder. See [Marketboard](marketboard.md). |
| MarketBoardPurchase | server → client | 353 | 665 (09.15), 610 (10.06) | 16 | A purchase went through: item and quantity. |
| RequestPlayerRetainerInfo | client → server | 348 | 580 (09.15), 818 (10.06) | 8 | Asks for the character's retainers as a retainer's sale list opens. See [Retainers](retainers.md). |
| RequestRetainerSaleHistory | client → server | 372 | 243 (09.15), 274 (10.06) | 8 | Asks for a retainer's sale history (their ID). |
| SetRetainerAppearance | client → server | 463 | 670 (10.06) | 32 | The new retainer's voice and CustomizeData from the vocate's editor. Finder. |
| RetainerStateNotice | server → client | 987 | 897 (09.15), 144 (10.06) | 8 | Before every RetainerState and after a retainer's containers. |
| RetainerState | server → client | 735 | 908 (09.15), 249 (10.06) | 56 | A retainer called (4), sent away (3) or hired (1). |
| PlayerRetainerInfo | server → client | 715 | 599 (09.15), 712 (10.06) | 480 | The character's retainers for the marketboard. |
| ItemMarketBoardInfo | server → client | 471 | 313 (09.15), 387 (10.06) | 32 | The asking price of an item a retainer has on the market. |
| ItemMarketBoardSummary | server → client | 290 | 442 (09.15), 303 (10.06) | 8 | How many items a retainer has on the market. |
| RetainerSaleHistory | server → client | 551 | 161 (09.15), 832 (10.06) | 1048 | A retainer's sale history. |
| LootMessage | server → client | 154 | 351 (09.15), 190 (10.06) | 32 | Loot: kind 3 an item obtained directly (an enemy's drop), 9 gil, 0/1 need/greed rolls and winners. See [Combat](combat.md#defeating-enemies). |
| ActionRequestGroundTargeted | client → server | 492 | 891 (09.15), 846 (10.06) | 24 | A ground-targeted action (Asylum...): the action, its type, the sequence, the rotation, where on the ground. Not yet seen on Kawari; the layout is from Bob's captures. Handled as an action on oneself for now. Finder. |
| Achievements | server → client | 985 | 991 (10.06) | 552 | The completed achievements and the last five, the reply to client trigger 1001. Finder: its handler (HandleAchievementsPacket). See [Achievements](character-windows.md). |

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
  561 were taken. A 7.57h1 recording would say which are retail's.

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
4. Write a finder so the next patch finds it automatically, and test it with `kawari-opcodescan`.

## After a game patch

1. Merge upstream (their `opcodes.yml` update).
2. Bump nothing in `fork_opcodes.yml` by hand: start the server against the new game; the log lists which fork opcodes
   were found by the scan and what to save. Update `opcode` and `found_on`.
3. Anything not found stays outstanding (safe placeholder) until found by hand.
