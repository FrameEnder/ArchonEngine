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

**The crash isn't explained yet.** Opening the window (Character → Portraits) crashes the client (Bob, 2026-10-16).
A zeroed reply says "no portrait", which is what the client asks the server for when it has none, so it isn't
obviously the cause. The window's addon is set up at `0x14658B0` (it names "BannerList"); the CharaCard getters
that hand out the adventurer plate's portrait are at `0xBC5540` (copies it, marking it present whenever the server
has answered) and `0xBC5970` (whether there is one). Without the faulting address (Dalamud's crash window or log,
`ffxiv_dx11.exe+0x…`) there are too many places to look.

## Recommendations

The Recommendations window (AgentRecommendList) is worked out by the **client**: it reads the RecommendContents sheet
against its own level and quest state, and refreshes when either changes; no server packet feeds it (the only thing
the client keeps for it is a "refresh needed" counter in UIState, which it raises itself after quests, level-ups and
similar). It becomes available with Spirithold Broken (129), whose client script prints its unlock message
(`LOGMESSAGE_RECOMMEND_LIST_UNLOCK`, LogMessage 3701) after the reward. So on Kawari it should list whatever the
client's quest and level state says; what's wrong with it there needs Bob's description.
