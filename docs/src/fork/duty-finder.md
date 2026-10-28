# Duty Finder Settings

What each of the Duty Finder's settings does once the party is in the duty, as retail plays them out. The leader's
settings come with QueueDuties (`DutyFinderSetting`, a u64 at its byte 8) and are kept with the duty's instance
(`Instance::content_settings`); the rules are in `servers/world/src/server/duty_finder.rs`, the daily reset in
`server/daily.rs`. Sources: the FFXIV Console Games Wiki's *Duty Finder*, *The Echo* and *Duty Roulette* pages, and
Bob's Chronofoil captures (66 scanned for this, 2026-10-28: numbers only).

Before Bob's bundle of 2026-10-28 ("I set my duty finder to unrestricted party ... the party I was already in wasn't
honored, and neither was the unrestricted party setting"), the settings came in and were sent back to the client,
level sync and Explorer Mode's shortcut aside nothing used them, and the party you had (three bots) was dropped at the
door and three new bots filled the duty.

| Setting | Flag | What it does here | Status |
|---|---|---|---|
| Your own party | | The bots in your party go in with you; the Duty Finder fills only the roles still missing, nothing when the party is full | done |
| Unrestricted Party | 0x2000 | The party goes in as it is (no bots added), unsynced unless Level Sync; no EXP, spiritbond or item drops from enemies when unsynced; Epic Echo | done |
| Level Sync | 0x200000 | With Unrestricted Party: synced to the duty's level (and its EXP and spiritbond again) | done (Kawari's existing sync) |
| Minimum IL | 0x4000 | Everyone's item level brought down to the duty's minimum, synced or not | done for players; not bots |
| Silence Echo | 0x10000000 | No Echo of any kind (the start's or a wipe's) | done |
| Explorer Mode | 0x100000000 | No enemies (none spawned by the zone or the director), no barriers, no bots added | done |
| Limited Leveling Roulette | 0x400000 | Leveling with a party of four: a duty within eight levels of the lowest member | done; QueueRoulette's layout assumed |
| Greed Only | 0x800 | Need counts as Greed | done |
| Lootmaster | 0x10000 | Played as the leader's "Greed only for the party" | partly (no lootmaster window) |
| Join Party in Progress, languages | 0x2, … | Nothing (there's no one else to match) | not done |

## Your party goes in with you

The bots in your party (`PartyBot::in_party`) now **travel with you**, into a duty and anywhere else: as you leave an
instance (`remove_actor_from_instance` → `party_bot::pack_for_travel`) each one is packed (its name, class, look,
glamour, content ID and how it's shown, as a kept friend is called again), and once you've loaded in where you went
(the zone-loaded handler → `party_bot::arrive`) they appear beside you, in the party, at that instance's synced level
(a level-synced duty's) or level 100, and the party list is sent. In a duty only as many come as its party has room
for besides you (none in a quest battle or an alliance's duty); the rest wait and come along at your next zone change.
Bots that follow you without being in the party stay behind, as before.

The Duty Finder then fills only what's missing: `fill_duty_party` takes the roles of the bots that came (and of Duty
Support's members) as already taken, so a party of a tank and a healer gets two DPS and a full party gets none. The
bots the Duty Finder added (`PartyBot::duty_filled`) and Duty Support's stay in the duty when you leave it (they're
dismissed, and the list empties as retail disbands a Duty Finder party), and your own bots come out with you.

A party of real players goes in as before, with no bots (bots can't join a party the server keeps).

## Unrestricted Party and Level Sync

* **No bots added** (`duty_finder::fills_the_party`): the party enters "with fewer players than the minimum",
  ignoring role requirements, with matching disabled (the wiki). Your own bots still come.
* **Unsynced unless Level Sync is on too** (Kawari's existing rule in `create_instance_for_content`): Level Sync is
  only offered with Unrestricted Party; without Unrestricted Party the duty is always synced.
* **Unsynced, nothing from enemies** (`duty_finder::unsynced_unrestricted`, `ZoneConnection::unsynced_unrestricted`):
  no kill EXP, no spiritbond (the gear still wears), no item drops (coffers are unaffected). Quest EXP and the rest
  still come. With Level Sync, EXP and spiritbond come again (the wiki: "allowing EXP and spiritbond gain").
* **Epic Echo** (status 2734): see below.

## Minimum IL

ContentFinderCondition.ItemLevelRequired is the duty's minimum (`GameData::find_content_minimum_item_level`); with
Minimum IL the instance keeps it (`Instance::minimum_item_level`) and each player is sent it as they load
(`FromServer::MinimumItemLevel`): their gear is capped at that item level (the same caps as a sync,
[Level Sync](level-sync.md#stats-under-sync)), synced or not, whichever of the sync's item level and the minimum is
lower. Retail's numbers under Minimum IL aren't recorded (no capture used it). Bots keep the gear of their level.

## The Echo and Epic Echo

Both raise maximum HP, damage dealt and the potency of healing (their Status rows). Retail (Bob's Chronofoil
captures 1aac5be2 and 61a9a4f3, a level 87 White Mage unsynced in A Realm Reborn's raids and extreme trials) gave
them with **param 0**, no duration, from 0xE0000000, only in the status lists, and PlayerStats' maximum HP already
raised at ZoneInit: the percentage is the server's own. Maximum HP there:

| Duties | Statuses | Max HP | × |
|---|---|---|---|
| CFC 101, 107–110 (Second Coil Turn 4, Final Coil) | 42 + 2734 | 167865 (34972 base) | 4.8 = 1.2 × 4 |
| CFC 97 (Binding Coil Turn 5) | 42 + 2734 | 177741 (34181 base) | 5.2 = 1.3 × 4 |
| CFC 78, 80, 95 (two extreme trials, Turn 3) | 2734 | 136724 | 4.0 |
| CFC 113–115, 136–139, 186–189 (Alexander, level 60) | 2734 | 104916 / 102543 | 3.0 |

which is the wiki's table: The Echo and Epic Echo multiply; Epic Echo 300 % for a level 71+ character in a duty of
levels 1–50, 200 % in 51–60.

* **Epic Echo** (`duty_finder::epic_echo_percent`): an unrestricted party without Level Sync, by the duty's level
  (its ClassJobLevelSync) and the highest member's (bots count, at 100): 30 % in a duty of their own level band, 60 %
  one band below them, 200 % two, 300 % three; none above level 80. Given as each player loads in.
* **The Echo from the start** (`duty_finder::echo_from_the_start`): 30 % in Turn 5 (97), 20 % in the Second Coil's
  Turn 4 and the Final Coil (101, 107–110), as recorded; none in Turn 3 (95). The wiki lists 10–30 % for the Coils and
  the older savage raids; the others aren't recorded, so they have none.
* **The Echo after a wipe** (`duty_finder::echo_on_wipe`, in `server/wipe.rs`): in a trial (ContentType 4) of an
  earlier expansion than the latest, a wipe after at least three minutes of fighting (`Instance::fight_started`: the
  first enemy hating one of the party) adds 10 % up to 50 %, in an extreme trial 5 % up to 25 % (the wiki). Extreme
  trials are told by their ContentUICategory ("High-end Trials", 19, 20, 22, 33, 39, 47): their HighEndDuty is false.
* **Silence Echo** turns all of it off.
* What it does (`duty_finder::refresh_echo`): each player and bot gets the statuses, a player's maximum HP is
  multiplied (`FromServer::EchoMultiplier`, their stats sent again), a bot's likewise, and the party's damage and
  healing are multiplied in `server/action.rs`.

## Explorer Mode

As retail (61a9a4f3 visit 29, the Dusk Vigil, CFC 36): InitDirector's third value 1 (Kawari already sent it), no
enemies (no SpawnNpc at all), no EXP or gil. The zone's battle NPCs were already left out as the instance was made;
now the director's (bosses, adds, `spawn_bnpc`) aren't spawned either (`Instance::insert_npc` turns hostile ones
away), its barriers are gone (Kawari's existing shortcut and walls), and no bots are added (your own come).

## Roulettes

* **Limited Leveling Roulette** (`duty_finder::limited_leveling_allows`, in `pick_roulette_duty`): Duty Roulette:
  Leveling with a party of four (you and three bots, by the party list's levels) rolls only duties whose required
  level is within eight of the lowest member's; otherwise no effect.
* **QueueRoulette's settings**: no QueueRoulette is in any capture, so where its settings are is assumed to be where
  QueueDuties has them (its byte 8; `unk1[7..15]`), and only Limited Leveling and the loot rules are taken from there
  (`duty_finder::roulette_settings`); the bytes are logged (`QueueRoulette …: bytes after the ID`). A bundle with a
  roulette queued would settle it.
* **The first clear of the day** (`ZoneConnection::roulette_cleared`, from DutyCleared when the duty was a
  roulette's): the roulette's gil, the wiki's level 90 amount (Leveling 7,770, Trials 12,000, Main Scenario 9,000…,
  `duty_finder::roulette_gil_at_90`) scaled by the level's QuestExpModifier against level 90's, with the quest
  reward's log line (1605); for Leveling also EXP, as a quest's (factor × ScaledQuestXP × QuestExpModifier / 100) with
  a factor of 1000 (`LEVELING_BONUS_EXP_FACTOR`: a stand-in, about a level at 16; retail's amount isn't in the game
  data or any capture). Tomestones (ContentRoulette's RewardTomeA–C) and Grand Company seals aren't given.
* **The daily reset** (`server/daily.rs`): 15:00 UTC every day, 8 AM in Arizona (no daylight saving), retail's
  (the wiki: 3 PM GMT). Each claim is kept in `daily-rewards.json` in the server's data folder (`$KAWARI_DATA`) with
  when it was made, and a reward is free again once a reset has passed since, so the day turns over at 15:00 UTC
  whether or not the server was up then.

## Loot rules

Greed Only and Lootmaster apply only when the duty allows them (its LootModeType's AdditionalLootRulesAvailable:
dungeons and extreme trials, not normal trials) and the party went in full, with no place for the Duty Finder to fill
(the wiki: "only fully premade parties can set these; random matches are always Normal") (`duty_finder::loot_rule`,
`Instance::loot_rule`). Greed Only: a Need counts as a Greed (`loot::choose`). Lootmaster: there's no lootmaster
window (its packets aren't known), so it plays out as the leader's other choice, Greed only for everyone. Bots pass on
everything as before ([Coffers and Loot](loot.md)). Nothing in the loot packets changed with the rule in the captures
(none used one).

## What retail sends that Kawari doesn't (yet)

From the same scan:

* **ContentFinderUpdate** came twice, 0.13–0.18 s apart, its settings a different set of flags than the client's
  (normal 0x40020040, unrestricted 0x60022040, explorer 0x140020060, Duty Support 0x48022048, in a party 0x20022024);
  Kawari sends the client's own flags back once.
* **ContentFinderFound** has the party's makeup at bytes 30–39 (`0000 0001 0001 0002` for a matched four, zeros and
  the party's size at the end for an unrestricted or premade party, byte 24 then 1).
* **A swift first-clear bonus**: Haukke Manor (CFC 6) gave 4,300 EXP and 3,800 gil at the end, with LogMessage 2163
  (the bonus for a swift first-time completion of the duty's objectives, its EXP and gil as parameters). Not done.
