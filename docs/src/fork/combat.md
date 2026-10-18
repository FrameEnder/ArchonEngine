# Combat and NPCs

What the fork changed about fighting: how damage is worked out, how enemies behave, what they give when they die, and
the White Mage kit. Numbers come from retail recordings (Kawari Bestiary, then Kawari Recorder) unless a line says
otherwise.

## Player stats and damage

**Stats** (`zone_connection/stats.rs`, `use_retail_formula`): a player's main stats start from the level's **main
modifier** (`level_main`, Sapphire's table, levels 1–10 checked against retail) instead of ParamGrow `BaseSpeed`.
Determination starts there too; Critical Hit and Direct Hit at `BaseSpeed`, as retail shows. Before this a level 7
Conjurer had 81 Mind (retail 35–38) and killed everything in one or two hits.

**Max HP** (`retail_max_hp`): `floor(base[level] × job HP modifier / 100) + floor(per_vit[level, tank] × (Vitality −
level main))`, the shape Akh Morning describes, with retail 7.x's numbers fitted from Bob's Chronofoil captures:

* his own PlayerStats (exact Vitality and HP: levels 1–22, 30–37, 50, 54, 60, 67, 70, 76, 80, 86–90 on Conjurer,
  White Mage, Samurai, Reaper, Gunbreaker, Paladin, Dark Knight, Black Mage, Pictomancer, Viper);
* level-synced party members (sync 50 and 80, 82 players): synced gear is capped, so a player's Vitality is the naked
  value (level main × the job's Vitality % + race) plus every slot's cap (ItemLevel[sync].Vitality × slot share ×
  MeldParam: 90 % for caster/healer gear, 100 % for the rest; [Level Sync](level-sync.md)). Their zone-in max HP
  (before the party bonus arrives, about 3 s later) matched to the point; the rest had food or were barely above the
  sync. All 112 of Bob's own points but the transition oddities match within 1 HP.

| Level | 1 | 10 | 18 | 22 | 30 | 36 | 50 | 54 | 60 | 70 | 76 | 80 | 86 | 90 | 100 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| base | 86 | 167 | 279 | 371 | 537 | 747 | 1400 | 1440 | 1500 | 1700 | 1880 | 2000 | 2600 | 3000 | 4000* |
| per Vitality | 5.6 | 5.6 | 5.6 | 5.6 | 8.0 | 9.0 | 11.2 | 11.9 | | 14.0 | | 18.8 | 22.1 | 24.3 | |
| tanks | | | | | | | 15.5 | | 17.5 | | 23.5 | 26.6 | | | |

(* extrapolated: +100 a level as from 80 to 90.) The base rises +9 a level to 10, +14 to 20, then faster; levels
between measured ones are interpolated. Tanks' per-Vitality outside 50–80 is the non-tank value × 15.5/11.2 (below)
or × 26.6/18.8 (above). Tanks are ClassJob Role 1. Checked by `retail_max_hp_matches_captures`. ParamGrow's
HpModifier and Sapphire's 5.x level table don't give these numbers.

**Regeneration** (`server/mod.rs`, every `PLAYER_REGEN_INTERVAL`, 3 s, as retail's tick): MP **200 a tick in combat**
plus Piety's bonus (`floor(150 × (Piety − the level's main stat) / level divisor)`: exact for all 17 of Bob's retail
points, e.g. 46 Piety at 14 → 203, 440 synced to 50 → 304, 1247 at 80 → 304; it used ParamGrow's BaseSpeed instead of
the main stat, 43 too little at 50), **600 out of combat**; HP **floor(max / 100) in combat** and **floor(max / 10)
out of it** (45 groups from level 6 to 90 exact). Retail keeps going while casting and ticks each actor on its own 3 s
phase (Kawari: one timer per instance). UpdateHpMpTp's third field is the max MP (10000 for players). Enemies out of
combat (and going home after giving up a chase) regain 10 % of their max HP a tick (149 retail samples). Measured in Bob's Chronofoil captures (2026-10-14/16): a conjurer casting
Stone (200 MP) every 2.5 s went +3 or −197 a tick, so barely drained at all. Kawari regenerated nothing in combat (and
everything every 500 ms out of it), so casters ran dry mid-fight and couldn't cast until it ended (2026-10-17 bundle).

**Damage and healing** use retail's formula as Sapphire has it:

```text
damage = potency / 100 × f(WD) × f(AP) × f(DET)

f(WD)  = floor(level_main × class modifier for the stat / 1000) + weapon damage (physical or magic)
f(AP)  = floor(125 × (attack power − level_main) / level_div + 100) / 100      (ATTACK_POWER_SCALE = 125)
f(DET) = floor(130 × (determination − level_main) / level_div + 1000) / 1000  (DETERMINATION_SCALE = 130)
auto-attacks: f(WD) × weapon delay / 3 s, potency 90
```

`level_div` is ParamGrow's `LevelModifier`. Attack power is Strength, or Dexterity for Dexterity classes; magic attack
is Intelligence, or **Mind for healers**; healing potency is Mind. Stone, Aero, Cure and auto-attacks land within a few
percent of the recording at levels 6–9. Traits are applied by the scripts (`whmPotency`, `classPotency`); variance,
criticals and statuses afterwards (below).

**Gear** (`GameData::equipped_item_stats`): an item gives its BaseParam values, defense and magic defense, plus its HQ
bonuses (`BaseParamValueSpecial`, item flag 1) when it's HQ. Checked against retail PlayerStats (Bob's captures, a
White Mage at 86 in HQ gear): Vitality 1757 = 370 naked + 1261 + 126 HQ, Mind 1885 = 424 + 1330 + 131, defense 1413 =
1267 + 146, every value to the point. Materia isn't counted yet. PlayerStats' `base_*` values are the **naked** stats at
the real level (retail: a level 6 Conjurer with 34 Mind sends 27; synced to 31, the unsynced naked 430), so the
character window shows the gear's bonus; `delay` is the weapon's (2400 for a Maple Wand). Under level sync items are
capped, see [Level Sync](level-sync.md).

### Damage and healing modifiers (`server/modifiers.rs`)

Applied after an action's script returns, by the server, so scripts only give the base amount and the statuses:

* **Player actions:** ±5 % variance (`PLAYER_VARIANCE`); a critical hit with the chance and multiplier from Critical
  Hit (`⌊200 × (CRIT − sub) / div + 50⌋ / 1000`, `⌊200 × (CRIT − sub) / div + 1400⌋ / 1000`: 5 % for ×1.4 at base),
  flag `0x20` on the damage effect (heals: `0x20` in their second byte); a direct hit from Direct Hit Rate
  (`⌊550 × (DH − sub) / div⌋ / 1000`, ×1.25, flag `0x40`). Retail (Bob's captures): ±5 % (Stone at 7: 19–22), crits 4 %
  at Conjurer 1–17 and 12 % at White Mage 80–88 for about ×1.35–1.5, direct hits almost never early on.
* **Statuses** by ID (the game's tooltip numbers): damage dealt Fight or Flight 76 +25 %, Raging Strikes 125 +15 %,
  Lance Charge 1864, Power Surge 2720, Surging Tempest 2677 +10 %; on an enemy Feint 1195 (−10 % physical, −5 % magic),
  Addle 1203 (the reverse), Reprisal 1193 −10 %; damage taken Rampart 1191 −20 %, Sentinel 74 −30 %, Sheltron 728
  −15 %, Vengeance 89 −30 %, Temperance 1872 −10 %, Trick Attack 3254 +10 %; Hallowed Ground 82 turns hits into
  "Invulnerable", Holmgang 409 keeps HP at 1 or more; healing received Mantra 102 +10 %, Thrill of Battle 87 +20 %,
  Rampart +15 %.
* **Tank Mastery:** Gladiators, Marauders and their jobs (and DRK/GNB) take 20 % less from enemies.
* **Enemy hits on players** keep their own variance and criticals (`roll_npc_hit`), then get the enemy's debuffs and the
  player's defences above.
* **Crowd control:** Stun (2) and Sleep (3) stop an enemy completely, Bind (13) keeps it in place; Sleep ends when it's
  hit (`npc_behavior`, `action.rs`).
* **Interrupts** (Head Graze, Interject) cancel only the target's cast and send ActorControl 15 (CancelCast) to everyone
  nearby; they used to delete every queued task of the target, so its statuses never wore off.

### Combos, animation lock, and actions without a script

* **Combos** (Bob's Chronofoil captures, 3,295 combo steps): only actions in a combo change it, those with an
  ActionCombo or that are another action's ActionCombo (`takes_part_in_combos`). Other spells and weaponskills in
  between don't break it (263 steps after Burst Strike, Inner Beast, Bloodspiller... kept full potency), and neither
  do abilities, items or mounts; a step of the same chain or another chain's starter does. The window is 30 s. Every
  action gets the ExecuteCombo effect (kind 27, value the action, param 128) with 1 when it continued a combo and 0
  otherwise; a combo step used out of its combo gets none. (Kawari only broke combos on other spells/weaponskills'
  ActionCategory before, and put a running counter in the effect.)
* **Animation lock** (ActionEffect): 0.1 after a spell with a cast, after auto-attacks and after mounting, 2.1 after
  Teleport and Return, 0.6 otherwise; and retail's own for some actions (`SPECIAL_ANIMATION_LOCKS`: 0.35, 0.5, 0.7,
  0.77, 0.8 or 1.5 s, e.g. Jump 0.8), every use the same in Bob's captures. Retail's items: medicines 1.1, food 2.1,
  items 4745–4751 3.1 (Kawari: 0.6).
* **Casts land 0.5 s early:** a player's cast action resolves half a second before its bar ends (retail 0.50–0.54 s
  for 4,165 of 4,805 casts of 1.1–7.7 s, Teleport too; mounts and items at the end). Moving before it lands
  interrupts it; after, it doesn't (that's slidecasting; Kawari's separate 0.5 s window is gone).
* **Recast** (ActorControlSelf 17, `SetCooldownTimerMax`: CooldownGroup − 1, the action, the recast in 10 ms units)
  is sent with every action a player uses, as retail does with the cast bar or the effect: for the GCD (group 58)
  Recast100ms sped up like a cast time (`1000 − floor(130 × (speed − BaseSpeed) / LevelModifier)` per mille, then
  haste), otherwise Recast100ms × charges. Exact for 5,394 of 6,043 retail GCDs (the misses: one synced Spell Speed
  where retail is 1 lower, and an action sending the 2.5 s group timer.)
* **Auto-attacks** come every weapon delay × haste / 100 (2 s without a weapon); they were every 2 s for everyone.
  Retail: Paladin 2.27 s for a 2.24 weapon, Warrior 3.38–3.40 for 3.36, Monk 2.05 (2.56 × 0.8 with Greased Lightning),
  Bob's 2.41 for 2.4.
* **Over-time ticks:** a damage tick is ActorControl **1541** (0, amount, source, 0xFFFFFFFF), a heal tick **1540**
  (0, amount, source, 1 if critical), each ±5 %; heal ticks went out as 1540 for both with the status ID. Retail adds
  all of a target's DoTs into one packet (and HoTs into another), ticks 1.5 s after the target's regeneration tick on
  its own 3 s phase, the first on the next such tick (0–4.5 s after the action, not a fixed 3 s), and scales ticks by
  Spell Speed (`1 + floor(130 × (speed − sub) / div) / 1000`); Lucid Dreaming's MP is ActorControl 23 (0, 11, 550,
  source). Kawari ticks each effect 3 s apart from when it was applied.
* An action **without a script** still gets an ActionEffect (with no effects) so the client's hotbar doesn't hang; the
  "Effect N isn't scripted" message for statuses is debug-level now (most statuses need no script).
* **Statuses** in StatusEffectList carry the time **left** (retail counts it down); Kawari sent each status's starting
  duration every time, so timers jumped back whenever the list was re-sent. Refreshing a status takes its new param and
  source, and the source (the caster) is filled in.

## Enemy damage to players

Enemies still compute damage like players do (stat × potency), which is far too high, so it is scaled
(`server/action.rs`, `npc_damage_scale`):

* If the enemy's **timeline** records its retail auto-attack damage by level (`damage_to_players` in
  `resources/timelines/<Name>_<BNpcBase>.json`), its auto-attack hits for that, and its other actions for that ×
  their potency / 100. Other levels scale with the per-level HP table. Filled in for the Central Shroud monsters.
* Otherwise it deals 12 % (`NPC_DAMAGE_TO_PLAYERS_PERCENT`). Retail's enemy formula isn't published. In Bob's
  captures (2,438 hits on him) an enemy's damage is set per enemy kind, not by level (level 79 ones hit for 967, 987 or
  1418); at equal level an open-world auto-attack is about 4–6 % of the enemy's own max HP, dungeon trash 1.7–2.7 %
  on a healer and 0.9–1.3 % on a tank, boss auto-attacks 6–12 % of a tank's HP.
* Every hit on a player varies ±5 %, crits 5 % of the time for 1.5× (retail 6.0 %, ×1.50), is parried 5 % for 83 %
  damage (retail 5–6 % on Bob, 7–9 % on tanks, 0.83×), and misses by how far the player is above the enemy
  (`npc_miss_percent`, from the captures): 2 % at the same level or below it, 12 % 1–2 levels above, 25 % 3–10 above,
  40 % more (retail 0–3 %, 12–14 %, 21–30 %, 32–53 %). It was a flat 8 %.
* **Recorded amounts** come first: a timeline's `recorded_amounts` (`{"<action>": {"players": avg, "npcs": avg}}`)
  says what that NPC's action did on retail to a player and to another NPC. The hit is scaled to it (then varied,
  missed, parried or critical as above), and a heal (Cure) is set to it ±5 %. Used for quest battle allies and
  enemies (Leia's Legacy, To Guard a Guardian), so they wear each other down, and heal, as fast as on retail.
* NPCs hitting other NPCs without a recorded amount are not scaled.
* Drains (`heal_self` effect, e.g. Blood Drain) heal the attacker by what they took; the attacker's HP is sent after.

Timeline fields the fork uses: `hitbox_radius` (reach includes it), `autoattack_action_id` (870/871/872 per kind),
`attack_range` (default 5 yalms; Papalymo 10), `damage_to_players` (`{"level": damage}`), `recorded_amounts` (above),
and `timepoints` (the
skills an enemy uses in turn). A trailing no-op timepoint keeps the real ones evenly spaced because the loop hits its
last point half as often. Sprites, roselets and floating eyes cast their spell instead of auto-attacking.

### Reading NPC-on-NPC damage from a recording

The Recorder's Monsters tab only keeps hits on the player, but its duty capture (`content_packets.json`) keeps every
ActionEffect1 (189) inside a duty, so who hit whom for how much can be read from it. Two fields are obfuscated per
session: the **action ID** has a key added (25 and 183 in the two 2026-10-16 runs: find it as the offset that turns
the most actions into enemy auto-attacks, 870–873), and each effect's **amount** (u16 at byte 6 of the 8-byte effect)
is **XORed** with a key, which is what the empty effect slots hold (`0xF8C5`, `0x843B`). Checked against the
Recorder's own damage-to-player numbers. The layout from the IPC data: animation target (8), action ID (8), global
sequence (12), …, effects from byte 42 (8 × 8: kind, then the amount at +6), target ID at byte 112. Targets are
told apart by their maximum HP (EffectResultBasic, 549: target at byte 8, HP at 12); attackers by which action they
use (e.g. 873 on Kuplu Kopo can only be the hedge knights), and one action two NPCs share splits into two clear
clusters (Galfrid's auto-attack on Quzal ~76, the bow's ~12).

## Enemy health

Enemies without a set HP use Sapphire's per-level **BnpcBaseHp** table (`BNPC_BASE_HP` in `server/actor.rs`, 44 at
level 1 to 2589 at level 50); it matched retail at every level recorded except 43 (retail 1891). A drop-in's `hp`
overrides it (Halitostroper 7983, Wood Wailer Sentries 6100, quest enemies).

Bob's Chronofoil captures (24,644 enemy spawns) show retail's open-world HP is one value per level **per expansion**
(7,540 of 8,026 ordinary spawns; no (zone, kind, level) with two values), so drop-in enemies without an `hp` get their
zone's (`field_enemy_base_hp`, by TerritoryType's ExVersion):

* A Realm Reborn: the table above, exact at every level seen (1–50; 42 showed 1341 on three spawns only).
* Heavensward, from 50: 6679, 7320, 7939, 8525, 9084, 9647, 9980, 10394, 10800, 11072, 11068 (60).
* Shadowbringers, 70–80: 30677, 34907, 35942, 36868, (74 interpolated 37673), 38478, 39129, 40048, 40901, 41915,
  42687 (also level 80 dungeons' plain trash).
* No Stormblood or Endwalker-and-later field zones were captured. Elites and city guards are exactly ×5 (guards at 35
  = 6100), a few ×2; FATE enemies are 3–0 levels below the FATE, mostly ×1 with ×0.62–×2 variants, bosses ×10–×36.
* In duties HP is set per enemy (491 kinds, one value each): trash 1.7–6× the field table at its level, bosses
  10–200×, levels the duty's sync or 2–3 below. The captures' values (by duty, BNpcBase and name) are in the scan's
  `duty_enemy_hp.csv` (not in git; numbers only) for drop-ins.

## Open-world spawns

Kawari only spawns battle NPCs that a zone's drop-in lists (`resources/dropins/`, matched to the zone by `appends`,
the LGB they add to): retail's LGBs don't have them.

| Zone | Drop-in | Where it came from |
|---|---|---|
| Central Shroud (148) | `fst_f1_planmap.json`, layer `LVD_kakashi` (834) | upstream, HP from the per-level table |
| Central Thanalan (141) | `wil_w1_planmap.json`, layer `BNPCS_RETAIL` (632) | Bob's Chronofoil sessions of 2026-10-17 (flying around the zone): `questkit chronofoil --dropin 141` |

Central Thanalan's spawn points carry retail's level and HP for each (levels 1–17, plus four level 35 Ul'dah guards at
their posts and one level 50 hunt mark, Ovjang, kept to one of its spots), whether it's aggressive (97 of 632) and its
links. The 21 FATE monsters seen were left out; their FATEs spawn them. Timelines (hitbox, auto-attack, damage to
players) were added for the 17 kinds that had none, from the Recorder export of the same day: every hitbox is
retail's; the auto-attack is the one seen in the captures for the Sun Bat, Antling Soldier, Snapping Shrew, Star
Marmot, Huge Hornet and Giant Tortoise (the rest keep the default 870); damage to players for the Snapping Shrew,
Star Marmot, Huge Hornet and Giant Tortoise; the Star Marmot also uses Somersault Slash.

## Enemy behaviour (`server/npc_behavior.rs`)

| Mechanic | How it works |
|---|---|
| **Enmity** | Each NPC keeps an enmity table. Damage adds to it; the NPC attacks whoever it hates most. Directors add enmity with `set_hate`. Before this, NPCs switched to whoever hit them last. |
| **Auto-attacks** | Every 3 s (retail 3.03 s over 8,200; a few kinds 4 or 5 s), timed per NPC from when it picked up its target. With a cast time they show a cast bar. |
| **Cast bars** | An NPC's ActorCast shows the action's cast time minus 0.3 s, and the action still lands at the full time: retail's for 99 % of 8,392 enemy casts (`npc_cast_bar_time`). |
| **Speed** | Running (chasing, going home) 6 yalms a second, walking about while idle 2 (retail's ActorMove speed byte, tenths of a yalm a second: 60 and 20; a few kinds chase at 3.) It was 4 for everything, always with the running animation. Players walk at 24 and run at 60 (Sprint 78, Peloton 72, ground mounts 90–150). |
| **Moving** | An NPC is only sent an ActorMove when it moved or turned. Sending one every half second to where it already stood (as reaching its goal and facing its target did) made the client play a step each time: idle NPCs flickered between walking and standing, and out in the world each sent about 1400 a session for a handful of real moves. In content an NPC with no path at all to its target (e.g. starting off the navmesh) walks straight at it instead of dropping it; dropping it sent To Guard a Guardian's second wave wandering back around the tree. |
| **Stance** | NPCs draw their weapon when they pick up a target and again 0.5 s after each action lands; they sheathe 2 s after losing their target. |
| **Placement** (`server/placement.rs`) | An NPC given `keep_near` (anchors and a distance, by the director) heads back to the middle of its anchors whenever it's further than that from any of them, whatever it's fighting, and attacks again once back. One given `stand_behind` (a tank and a distance) goes to the spot that far beyond the enemy the tank is fighting, on the far side from the tank (out of the enemy's frontal cones), when that spot keeps it near its anchors and its own target is in reach from there; within 2.5 yalms counts as there. |
| **Pathing** | A new path starts from where the NPC actually is, and changing target drops the old path (no more 14–16 yalm jumps). When the navmesh doesn't reach the target, Detour returns a *partial* path (status `0x40000040`, success + partial, which used to be logged as an "Error in dtNavMeshQuery_findPath"): the NPC walks it and then straight on to the target, if that last stretch is at most 15 yalms (`MAX_OFF_MESH_GAP`), and in content however far. Before, it stopped at the edge for good, out of range (To Guard a Guardian's allies stuck 9 yalms from two Ixali, so the battle never ended, 2026-10-16 bundle). |
| **Aggro** | Aggressive enemies notice players within 11 yalms all round (`AGGRO_PROXIMITY`) or within 15 yalms in a ±45° cone in front of them (`AGGRO_RANGE`), while idle *or* wandering, and ignore players 9+ levels above them (`AGGRO_LEVEL_GAP`). From Bob's captures: 98 enemies came at him on their own from 2–18 yalms (median 9); the far ones (12–18) all faced him, every one from behind was within about 11; they still did at 7–8 levels below him (8 of 41 close passes), never at 9+ (0 of 154). (Retail's assist, idle enemies of the same kind within about 4–14 yalms joining when one is hit, isn't done.) |
| **Leashing** | Chased more than 40 yalms from spawn (`LEASH_DISTANCE`; retail gave up at 34–45, median 39, after 7–20 s, home 7–9 s later at a run), or once the one it's after is that far from its spawn, an enemy drops all enmity, regenerates on the way (10 % a tick; it used to heal to full at once) and walks home ignoring everyone until within 3 yalms. Enemies wander around their spawn, **only out in the world**: in content NPCs stand where they're put until something starts them (To Guard a Guardian's masked mage walked off while waiting to be hit). (The second rule is for enemies with no navmesh path to the player: they never moved, so never leashed, and kept the player in combat anywhere in the zone, 2026-10-16 bundle.) **Only out in the world**: enemies in content (a quest battle, a duty) never leash. In To Guard a Guardian the second wave's lostwings, sent the long way round after Miraudont, leashed at full health, and back home picked up a background guard instead, which they could never kill. |
| **Claim** (`server/first_attack.rs`) | The first to attack an actor claims it: FirstAttack (374 on 7.57, 16 bytes: kind u32, 0, the claimer's ID u32, 0) goes to everyone nearby with the attacked actor as its source, just before the action's EffectResult, when it first takes damage or a miss: an enemy from a player or an allied NPC, a player from an enemy. It stays with the first attacker until they're gone (dead or left). Cleared (kind 0, no ID) for an enemy that gives up a chase and for a player whose fight is over (as InCombat goes; on retail 0 to 3 s after their enemy list empties). The claimer is also CommonSpawn's `combat_tagger_id`, so others see it when the actor spawns for them. From Bob's Chronofoil captures (2026-10-14 to 17): about 1,100 of them in the Central Shroud and Thanalan, every set one right before the target's EffectResult. Kawari never sent it before. |
| **In combat** | While any enemy has a player on its enmity list (the HaterList), the server sets the player's **InCombat** condition (wire bit 1, client condition 26), as retail does, and clears it when none do. The client then refuses Teleport, Return, mounting and talking to NPCs itself. Natural HP/MP regeneration waits for it to clear. Before this the server never set it, so the client let players teleport with enemies still after them. |
| **Area attacks** | Cones, circles and lines from the Action sheet (`CastType`, `EffectRange`, `XAxisModifier`; cone angle from the omen, e.g. `gl_fan090` = 90°). Aimed where the target was and the way the enemy faced when the cast started; the enemy stands still while casting; ground-targeted omens show where the target stood. They land only on players still inside when the cast ends, counting half a yalm past the edge (`PLAYER_HITBOX_RADIUS`, the player's hitbox; it was a whole yalm, so a player who had left a 6-yalm Terror Eye was still hit at 6.6 yalms), each getting their own roll (damage, variance, miss/parry/crit, statuses). The ActionEffect packet still shows only the main target (only the one-target opcode is known). |
| **Striking dummies** | Keep attackers on their enmity list but don't move, turn or fight back; resetting enmity clears it. |

Player side: sheathing stops auto-attacking (the client doesn't always ask).

## Class rotations (`server/rotation.rs`)

NPCs in duties fight with their class's actions (Miraudont is a lancer, Yda a pugilist, Papalymo a thaumaturge), so a
timeline can give an NPC a **rotation** instead of fixed timepoints. The class mechanics are read from the Action sheet
at runtime (`GameData::get_action_mechanics`): whether an action is on the GCD (CooldownGroup 58, or as its additional
group), its recast, and the action it follows in a combo (`ActionCombo`). The rotation only adds what retail's
recordings show, and `questkit battle` writes it from a Recorder 2.7 recording:

```json
"rotation": {
  "pace": 4.0,
  "actions": [
    { "action": 83, "then": 78, "comment": "Life Surge: on cooldown, then Vorpal Thrust" },
    { "action": 82, "comment": "Leg Sweep: on cooldown" },
    { "action": 76, "every": 40, "comment": "Feint: every 40 s, not on cooldown" },
    { "action": 78, "comment": "Vorpal Thrust: only right after True Thrust (its ActionCombo)" },
    { "action": 75 }
  ]
}
```

Every tick (500 ms) the first ready entry is used:

| Rule | |
|---|---|
| `pace` | Seconds between the NPC's weaponskills and spells. Retail NPCs act far less often than players (allies use one every 3–12 s). |
| GCD actions | Only once `pace` has passed; a combo step only right after its `ActionCombo`, within 15 s, when that step is in the rotation too (retail NPCs whose kit lacks it use the later step on its own: Underneath the Sultantree's stranger and Rage of Halone); one with a longer recast than `pace` waits for it. |
| Abilities | Between two GCD actions only (never before the first), a second after the last one, one at a time; then their recast (or `every`). |
| `refresh` | Not again on the same target for that many seconds: a damage-over-time or debuff, re-applied as it wears off (timed per target, since players' statuses aren't visible here). |
| `then` | The GCD action that comes next (a buff's follow-up). |
| `target_hp_below`, `self_hp_below` | Only below that health %. |

A rotation's action replaces that tick's auto-attack; casts work as for any NPC action (it stands still while casting).
Heals on allies stay in the content's Lua (`runHeals`), since they pick a target other than the NPC's own.

## Defeating enemies

When a battle NPC dies, everyone on its enmity list, and their party in the same instance, gets:

* **Quest credit**: each of their active quests' `onBNpcKill(base_id, layout_id, player)` (see [Quests](quests.md)).
* **EXP**: ParamGrow's `ScaledQuestXP` for the enemy's level (= 45 + 5 × level, exactly what retail gave). **Kill
  chains** for enemies at or above your level within 60 s: +20 %, +25 %, then +30 %. **Rested EXP** adds 50 % to kill
  and gathering EXP only (a rounding bug once made it +100 %). Bonuses show in the floating EXP.
* **Drops** (`resources/scripts/battle/Drops.lua`, by BNpcName): items from Garland Tools' data; chance and amount
  from recordings where known (anole 67 % ×3, ground squirrel 17 % ×2, trickster imp 11 %), otherwise one 50 % roll
  (a guess). Central Shroud only so far.

Level-ups announce only the new levels (the message's third value is the previous level).

**Death and respawn** (Bob's captures, 1,581 deaths): the death animation (Kill), then **Defeated** (ActorControl 6,
the killer twice, an invalid ID for none) to everyone nearby 2.0 s later (Kawari: the one it hated most stands in for
the killer; it never sent Defeated before), DeadFadeOut 8.05 s after the death, DeleteActor 1.75 s after that (was 8 +
2 s). Respawns count from the deletion, in retail's steps by level (`mob_respawn_time`): 15.7 s for levels 1–4, 30.9 s
for 5–9, 61 s from 10 (retail also used 120.7 s at some spots of any level; Kawari used 120 s for everything).

## Healer engine and the White Mage kit

From the first fork merge (`dc63634`):

* Heals restore HP (Benediction heals to full). Spells cost MP from the Action sheet; Thin Air, Freecure and Swiftcast
  are consumed; Swiftcast makes the next cast instant and Presence of Mind cuts cast times by 20 %, on top of upstream's
  adjusted cast times.
* Area actions hit every enemy or party member in range. DoTs, HoTs and MP-over-time tick every 3 s.
* The Lily gauge (Lilies, Blood Lily) with `spend_lily` / `spend_blood_lily`.
* Action scripts can see the caster's statuses (`get_effect`); refreshing a status no longer makes it expire early.
* All 36 Conjurer/White Mage actions and 6 role actions (`resources/scripts/actions/`), with 7.56 potencies;
  Global.lua has their status/trait IDs and `whmPotency` (Maim and Mend +10 %/+30 %) and `traitPotency`.
* Fixed: every action wiped the instance's queued tasks (an inverted `retain`).

Enemy skills added for the Central Shroud and the quest battles: Frost Breath, Ice Spikes, Blood Drain, Acorn
Armageddon (Heavy ~14 s), True Grit, Plaincracker, Hammer Beak, Snap Jaw and others (see `actions/003`–`actions/009`),
plus True Strike, Snap Punch, Fire, Thunder for the quest battle allies, and Bloody Claw, Rage of Halone and Spirits
Within for Underneath the Sultantree.

## Class and role actions, levels 1–50

Every action from level 1 to 50 of Gladiator/Paladin, Pugilist/Monk, Marauder/Warrior, Lancer/Dragoon, Archer/Bard,
Thaumaturge/Black Mage, Arcanist/Summoner, Scholar and Rogue/Ninja has a script now (131 new, 26 rewritten; Conjurer/White
Mage was already complete), plus the role actions up to 50 (Rampart, Provoke, Reprisal, Shirk, Interject, Low Blow,
Second Wind, Bloodbath, True North, Addle, Leg Sweep, Sleep, Rescue, Esuna). How they were checked:

* **Retail data:** Bob's captures only have Conjurer/White Mage player actions (and the healer role actions), so the
  others are from the 7.x game data: potencies are the tooltip text (the **max-level** values; each script says so, and
  the client's tooltip on Kawari shows the value at your level to compare), status IDs and durations from the Status
  sheet. 7.x player IDs where they differ from the old ones NPCs still use: Blizzard II 25793, Thunder II 7447, Wide
  Volley 36974, Leg Sweep 7863.
* **Shared IDs:** 18 actions are also used by quest battle NPCs (9, 53, 54, 56, 70, 75, 78, 97, 100, 106, 113, 141,
  142, 144, 147, 154, 156): their scripts keep the recorded NPC numbers unless `isPlayerCaster(player)` (the new
  `player.is_player`).
* **Combo steps** use `in_combo` for their combo potency and bonus (Riot Blade 330 and MP, Maim, Storm's Path/Eye, Vorpal
  Thrust, Gust Slash...). Traits: `classPotency(player, potency, TRAITS_...)` for Thaumaturge and Arcanist Maim and Mend
  and Archer's Increased Action Damage.
* **New engine methods:** `effects:provoke()` (top of the target's enmity), `effects:cleanse()` (removes the target's
  first status with Status.CanDispel); players' `player.is_player`, `hp`, `max_hp`, `mp`, `max_mp` for scripts.
* **Second Wind** is potency 500 below level 94 (it was 800).
* **Not there:** job gauges (Black Mage's Astral Fire/Umbral Ice — Thunderhead stands in so Thunder can be cast —,
  Monk's chakra and furies, Warrior's Beast Gauge, Paladin's Oath, Summoner/Scholar Aetherflow, Ninja mudras: Ninjutsu
  is always Fuma Shuriken), pets other than Carbuncle (no egis, no Eos), positionals, movement actions (Thunderclap,
  Rescue...), barriers absorbing damage, Bloodbath's lifesteal, Peloton for the party. See
  [Known Issues](known-issues.md#combat).

**Combat items** (`items/Items.lua`): potions (ItemAction 847: the % of max HP up to the cap), ethers (848, MP),
elixirs (849, their caps), status removal (842), food (844–846: Well Fed 48 with the ItemFood row and duration, no stat
bonus yet).

## Adding a monster or skill

1. Record it (Kawari Recorder → Monsters tab: level, HP per level, skills, damage to players at your level).
2. Timeline: `resources/timelines/<Name>_<BNpcBase>.json` with its hitbox, auto-attack, `damage_to_players`,
   timepoints.
3. Skill script: `resources/scripts/actions/<id / 100>/<Name>_<id>.lua`, potency relative to its auto-attack.
4. Drops: `Drops.lua` by BNpcName.
5. Fixed spawns: the zone's drop-in.
