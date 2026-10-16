# Achievements, Portraits and Recommendations

Three of the character windows Bob tried on retail (2026-10-16) and found broken or missing on Kawari. What the
client asks for and expects comes from reading `ffxiv_dx11.exe` (2026.10.06) beside FFXIVClientStructs; the
Recorder doesn't capture these packets (it only keeps packets addressed to quests and directors), so nothing here is
checked against a retail capture yet.

## Achievements (`servers/world/src/zone_connection/achievement.rs`)

**What the client does.** The Achievements window asks with client trigger **1001** (Achievement::
RequestCompletedAchievements, `ClientTriggerCommand::RequestAchievements`) and stays on "loading" until the reply, the
**Achievements** packet (server opcode **991**, 552 bytes, `fork_opcodes.yml`; FFXIVClientStructs' AchievementsPacket):

| Offset | Size | Contents |
|---|---|---|
| 0x000 | 510 | Completed achievements, Achievement sheet rows, **lowest bit first** (row 1 is bit 1 of byte 0). IDs above 4077 are ignored. |
| 0x1FE | 10 | The last five completed (u16), newest first. |
| 0x208 | 26 | Unknown (the client copies it beside its near-completion data); sent as zeroes. |

Other requests and replies:

| Client | Server | Notes |
|---|---|---|
| trigger **1000** `RequestAchievementProgress { id }` | actor control **514** `AchievementProgressInfo { id, current, max }` | The counter shown under an achievement. |
| trigger **1002** `RequestNearCompletionAchievements { slots }` (bit 0: the login notice, bit 1: the window's Near Completion page) | actor control **538** with the slot in its first parameter | 538 empties that list and marks it loaded; the server sends it for each requested slot, so both lists are empty for now. |
| | actor control **515** `AchievementCompleted { id }` | The client marks it complete, adds it to its history and shows the "Achievement unlocked" pop-up. |
| | actor control **600** | Marks the first parameter complete when the second is 1, and takes the third and fourth as its progress. Not used. |

The actor control cases were read from the client's actor control switch (`0xB3A2E0`, jump table at `0xB47964`):
514 calls ReceiveAchievementProgress, 515 SetAchievementCompleted and the pop-up queue, 538
ReceiveNearCompletionAchievements with no bitmap.

**What the server keeps.** `unlock.achievements` (510 bytes) and `unlock.achievement_history` (five little-endian
u16, newest first), migration `update-2026-10-16_achievements_portrait`, sizes `ACHIEVEMENT_BITMASK_SIZE` and
`ACHIEVEMENT_HISTORY_SIZE` in `constants.yml`.

**Completing them.** After finishing a quest and after levelling up, the server completes (with the pop-up) every
achievement whose requirements are met, for the Achievement sheet Types it understands. What each Type means comes
from the community's sheet definitions (SaintCoinach), not from the client:

| Type | Needs | Progress shown |
|---|---|---|
| 2 | Every achievement listed in Data | done / listed |
| 3 | The class or job in Data[0] at level Data[1] | level / wanted |
| 6 | Every quest listed in Data | done / listed |
| 9 | Any quest listed in Data | 0 or 1 of 1 |

Everything else (kill, craft, gathering counters, dungeons, Grand Company ranks…) isn't tracked; those can be given by
hand with `!achievement <id>`. `!achievement clear <id>` takes one away, `!achievement check` runs the check (for
characters that met requirements before this existed). The sheet is read once, the first time it's needed.

## Portraits

**What the client does.** Client trigger **3200** (Kawari's `OpenPortraitsWindow`) asks for the **adventurer plate's
portrait**; the client sends it whenever it doesn't have it yet, e.g. opening the Portraits window. The reply,
**PortraitsInformation** (182, 56 bytes), is copied into CharaCard's CurrentBannerData (FFXIVClientStructs'
BannerData, 52 bytes: a "has one" byte, expression, lighting colours and brightness, then u16s for the banner pose,
its progress, head and eye direction, camera position and target, image rotation, light angles, decoration,
background and frame, and a u32 checksum of the gear) and marks it received. Setting a portrait as the adventurer
plate's sends **SetAdventurerPlatePortrait** (client opcode **107**, 56 bytes, the same 52 bytes) — a fork opcode,
found by the function that sends it.

The portraits themselves (the list in the window) are the client's own: BannerModule keeps them in the character's
local UI save data, not on the server.

**What the server does now.** Saves the 52 bytes it's sent in `volatile.portrait` (same migration) and sends them back
for 3200; a character who never set one gets zeroes, as before.

**The "crash" was the server.** Opening the window (Character → Portraits) disconnected the client with error 90002
("Connection with the server was lost"), because the world server panicked: the window asks for the player's own
adventurer plate (RequestAdventurerPlate), and upstream's `lookup_adventurer_plate` (`database/character.rs`) read
the Grand Company rank as `company_ranks[active_company - 1]`, which underflows for a character in no Grand Company
(0). It now sends rank 0 then, as the other two rank lookups already did. (For reference, the window's addon is set
up at `0x14658B0`, naming "BannerList"; CharaCard's portrait getters are at `0xBC5540` and `0xBC5970`.)

## Recommendations

The list itself is worked out by the **client** (AgentRecommendList reads RecommendContents against its own level
and quest state); no server packet feeds it. But the window has to be **unlocked**: AgentRecommendList's
IsActivatable (`0xC9F340`) checks **unlock link 9** (UIState IsUnlockLinkUnlocked) and that main command 31 is
enabled. Unlock link 9 is Spirithold Broken's `REWARD0` quest parameter, which the server grants when the quest is
finished (see [Quests](quests.md#quest-rewards-and-exp)); until the fork did that, the main menu's entry was greyed
and the window answered "That command is unavailable at this time" (LogMessage 488).

## How the client decides a main menu command is available

UIModule's ExecuteMainCommand (vf206, `0x78BEE0`) prints LogMessage 488 whenever IsMainCommandUnlocked (vf207,
`0x78C5B0`) says no. For commands above 26 that looks the command up in a table at `0x28A8270` (16-byte entries:
main command, addon, agent ID, condition flags), asks the agent's IsActivatable (vf8), then refuses it under some
conditions when the flags say so. Entries: 31 Recommendations → agent 109 (flags 1), 92 Portraits → 407 (flags 1),
93 Adventurer Plate → 413 (flags 0). **AgentCharaCard's IsActivatable (`0xDC1220`) is only "not condition 56"**
(BoundByDuty56, Kawari's wire bit 28: the networked conditions are numbered in Condition sheet order, `IsNetworked`
rows only, so wire bit n is the n-th of those), so the Adventurer Plate is refused exactly when the client thinks it's
bound by duty.
