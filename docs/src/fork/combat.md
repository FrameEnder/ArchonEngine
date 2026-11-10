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

**Damage and healing** use retail's formula, fitted to Bob's Chronofoil captures (since 2026-11-10;
`zone_connection/stats.rs`, `retail_damage`):

```text
damage = floor(floor(potency × f(AP) × f(DET) / 100 / 1000) × f(WD) / 100)

f(AP)  = floor(c × (attack power − level_main) / level_main) + 100        (hundredths)
f(DET) = floor(140 × (determination − level_main) / level_div) + 1000      (thousandths)
f(WD)  = floor(level_main × class modifier for the stat / 1000) + weapon damage (physical or magic)
auto-attacks: f(WD) × weapon delay / 3 s, potency 90
c: damage 75 up to level 50, 165 at 80, 195 at 90, 237 at 100 (tanks 115, 156, 190 there); healing 75 up to 50,
   145.8 at 90; straight lines between (ATTACK_POWER_ANCHORS, TANK_ATTACK_POWER_ANCHORS, HEALING_POTENCY_ANCHORS)
```

`level_div` is ParamGrow's `LevelModifier`. Attack power is Strength, or Dexterity for Dexterity classes; magic attack
is Intelligence, or **Mind for healers**; healing potency is Mind. Traits are applied by the scripts (`whmPotency`,
`classPotency`); variance, criticals and statuses afterwards (below).

How it was found (`questkit chronofoil <captures> --hits hits.jsonl.gz`, [questkit](questkit.md)): 124 captures
(June to November 2026, Bob's characters and Connor's) gave about 24,000 of the players' own hits and heals with
their class, level, PlayerStats, main hand and the target's level. The weapon's damage isn't in a capture (only its
item ID), so each weapon's was fitted along with the rest: a weapon used over several levels pins it (40 of them,
level 1 to 45). Non-critical hits only, single targets at or below the player's level, and heals (no enemy in them):

* **Up to level 50, c is 75 over the level's main stat**, for damage and healing alike and for tanks too: Conjurer and
  White Mage Stone, Stone II, Cure, Medica and Cure II at 6-44 (8 weapons), Arcanist Ruin and Physick at 8-45 (8),
  Gladiator and Paladin Fast Blade and auto-attacks at 1-44 (10) fit within 1-2 % (heals 0.5 %, the Arcanist's 1 %).
  Sapphire's 125 over `level_div`, Kawari's before, fitted worse each time (heals 0.70 % to 0.58 %, the Arcanist 1.24 %
  to 1.03 %). The two differ by a few percent up to 50 (at level 10 125/78 against 75/35 of a point: 5 % less
  damage), but past it Sapphire's grows with `level_div` (1300 at 80, 1900 at 90) and fell far behind: a level 80
  White Mage's Glare with 1557 and 1933 Mind did 3533.5 and 4507.1 (1.2755 times; Sapphire's f(AP) 1.166, this one
  1.265), and a level 86-88 White Mage's Glare III 4700-4900 where Sapphire's gave about a third of it.
* **Past 50 healing grows more slowly than damage:** at 87 Cure II healed 2.0225 times what Glare III did with the
  same 1928 Mind (capture 281f92fb), which the anchors give (2.020); one f(AP) for both gave the potencies' 2.58.
  The anchors at 80, 90 and 100 are the community's (Allagan Studies); level 80's Glare and 87's heals agree with them,
  levels 51-79 and the tanks past 50 weren't recorded (interpolated).
* **Determination** 140 over `level_div` (retail's; 130 fitted as well), with the level divisor table checked against
  the MP ticks (Piety, below).
* **Potencies** that fit: Stone 140, Stone II 190, Cure 450, Medica 300, Cure II 700 (800 at 85), Physick 400, Fast
  Blade 150. Ruin 220 fitted a little better than the script's 240, but the Arcanist's Intelligence modifier trades
  off against it, so it's left as it is.
* **Variance** ±5 % on damage, about ±3.5 % on heals (Cure at 17: 104-111 around 107.3).
* **Quest battles' role statuses:** the same characters hit 1.25-1.31 times as hard in quest battles as elsewhere (tanks
  and healers), and healed the same: retail gives each player a status for their role as the battle begins (with
  scene 60000, in StatusEffectList alone, from no one, with no end): tanks 1264, healers 1041, the Arcanist 1265 (its
  damage unchanged). Kawari gives them (`director::QUEST_BATTLE_ROLE_STATUSES`) and 1264 and 1041 add 30 % to damage
  (`modifiers::DAMAGE_DEALT`). In The Eyes of Gods and Men's battle a level 40 White Mage's Stone II did 208-224 to
  the enemy conjurers, 1.29 times the formula's; Kawari's player did 130-150 without it (Bob's bundle
  20261110-081839).
* **Enemies above the player** took less: about 0.89-0.90 of it at one to three levels above, 0.86 at four or five,
  0.81 at six or seven (open world, a few hundred hits); Kawari doesn't do this yet ([Known Issues](known-issues.md)).

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
* **Tank Mastery:** Gladiators, Marauders and their jobs (and DRK/GNB) take 20 % less from enemies. Their heavier armour takes
  a further 17 % off (`TANK_ARMOUR`, 0.83, since 2026-11-10): the amounts Kawari has for enemies' hits on players were
  recorded on Bob's casters and healers, and in the Stone Vigil his Paladin (42, capture 341808b0) took a median 0.67
  of what his Arcanist (41, capture 1cefedf4) took from the same actions (0.49-0.78, ten actions), Tank Mastery's 0.8
  times 0.83. With Tank Mastery alone the party bots' tank took 1.2 times retail's (Bob's bundle 20261110-134659).
* **Enemy hits on players** keep their own variance and criticals (`roll_npc_hit`), then get the enemy's debuffs and the
  player's defences above.
* **Crowd control:** Stun (2) and Sleep (3) stop an enemy completely, Bind (13) keeps it in place; Sleep ends when it's
  hit (`npc_behavior`, `action.rs`). Petrification (1) stops an NPC too (since 2026-11-06: quest battle 61's Stone
  Gaze, 9 s; retail's petrified allies did nothing for it seven times in ten), as does the Navel's Granite Gaol (292).
  A **player** with one of these is held as retail held them (since 2026-11-09, `server/mod.rs`): the conditions
  SufferingStatusAffliction and SufferingStatusAffliction2, which stop the client moving and acting, and
  untargetable (ActorControl 54), until it goes ([Dungeons](dungeons.md#the-navel), the gaols).
* **Interrupts** (Head Graze, Interject) cancel only the target's cast and send ActorControl 15 (CancelCast) to everyone
  nearby; they used to delete every queued task of the target, so its statuses never wore off.

### Combos, animation lock, and actions without a script

* **Combos** (Bob's Chronofoil captures, 3,295 combo steps): only actions in a combo change it, those with an
  ActionCombo or that are another action's ActionCombo (`takes_part_in_combos`). Other spells and weaponskills in
  between don't break it (263 steps after Burst Strike, Inner Beast, Bloodspiller... kept full potency), and neither
  do abilities, items or mounts; a step of the same chain or another chain's starter does. The window is 30 s. Every
  player spell and weaponskill gets the ExecuteCombo effect (kind 27, value the action, param 128) with 1 when it continued a combo and 0
  otherwise, and abilities and auto-attacks none (Fight or Flight 0 of 20, Low Blow 0 of 12, Sprint 0 of 43: sent on Fight or Flight
  it made the client drop the Fast Blade → Riot Blade combo, 2026-10-20); a combo step used out of its combo gets none. (Kawari only broke combos on other spells/weaponskills'
  ActionCategory before, and put a running counter in the effect.)
* **Animation lock** (ActionEffect): 0.1 after a spell with a cast, after auto-attacks and after mounting, 2.1 after
  Teleport and Return, 0.6 otherwise; and retail's own for some actions (`SPECIAL_ANIMATION_LOCKS`: 0.35, 0.5, 0.7,
  0.77, 0.8 or 1.5 s, e.g. Jump 0.8), every use the same in Bob's captures. Retail's items: medicines 1.1, food 2.1,
  items 4745–4751 3.1 (Kawari: 0.6).
* **Each hit's result:** after an ActionEffect, the target sends its result as the hit lands: EffectResultBasic
  (opcode 235 on 7.57, 24 bytes: count 1, the ActionEffect's global sequence, the target, its HP, 8 zero bytes), or
  EffectResult when it gained statuses, both under **the ActionEffect's own global sequence**. Retail (Bob's
  Chronofoil capture of 2026-10-21): 6,603 of 6,838 results share their effect's sequence, sent 0.5 s after it (3,677
  of 6,180; 0.8–1.1 s for most others), and for a player's cast about 0.57 s after its bar ends (Ruin: 1.07 s after its
  early effect). The client holds an action's damage until its result comes: Kawari sent none for plain damage and
  gave EffectResults the next sequence, so each spell's damage only showed when the next spell's effect arrived (Bob,
  2026-10-21). Kawari now sends EffectResultBasic 0.5 s after the effect (`server/action.rs`, a queued `EffectResultLands`) to each target the effect showed on, unless it got an EffectResult. Its HP is the target's as the result goes out, not as the action went off: with several attackers, a result landing after other hits put back what they had taken and the next took it off again, the enemy's bar losing, gaining and losing health (169 times in one of Bob's Copperbell Mines runs, 2026-10-25).
* **Casts land as the bar ends:** on retail a player's ActionEffect comes half a second before its bar ends (0.50–0.54 s
  for 4,165 of 4,805 casts of 1.1–7.7 s, Teleport too; mounts and items at the end), but the target's side (FirstAttack,
  aggro, its health in EffectResultBasic, death) comes 1.07 s after that effect, about 0.57 s after the bar (Bob's
  Chronofoil capture of 2026-10-21, Ruin), and the target's UpdateHpMpTp isn't sent. Kawari applies an action's
  effects (health, death, EXP) the moment it sends them, so from 2026-10-18 to 2026-10-21 its casts landing early took
  the target's health and killed it before the bar had ended (Bob's 2026-10-21 bundle); they land when the bar ends
  again, the result 0.5 s later, and moving in the last half second doesn't interrupt them (slidecasting,
  `SLIDECAST_WINDOW_MS`). Summoning a mount isn't interrupted by moving at all ([Chocobo](chocobo.md#riding)).
* **Recast** (ActorControlSelf 17, `SetCooldownTimerMax`: CooldownGroup − 1, the action, the recast in 10 ms units)
  is sent with every action a player uses, as retail does with the cast bar or the effect. It's worked out as the
  client does (`client_rules.rs`, ported from ActionManager.GetAdjustedRecastTime and GetMaxCharges of the 2026.10.16
  client, unchanged in the 2026-10-20 build):
  * Recast100ms × 100, plus the learnt traits' Value × 100 (Enhanced Swiftcast's −20 s from its level; 19 actions);
  * for a spell or weaponskill with at most one charge at the player's level, sped up like a cast time
    (`1000 − floor(130 × (speed − BaseSpeed) / LevelModifier)` per mille, then haste), never under 1.5 s;
  * otherwise × the action's charges at the highest level: the timer (RecastDetail.Total) is the whole charge gauge,
    which the client's GetRecastTimeForGroup cuts down while the extra charges aren't learnt. Charges are 1 when the
    row's MaxCharges is 0, else the Value of the first learnt trait of the action's chain (Essential Dignity: Enhanced
    Essential Dignity II, then I), else MaxCharges (38 actions have a chain).

  A trait is learnt when its ClassJobCategory has the class, its Level is set and reached, and its Quest (none has one)
  is done. The action → trait tables are compiled into the client's code, not in a sheet:
  `python3 tools/clientlogic/clientlogic.py <ffxiv_dx11.exe> --rust` reads them (finding the two functions by
  signature and following their compare chains), for re-checking after a game update.
  Checked against retail: 5,394 of 6,043 GCDs exact (the misses: one synced Spell Speed where retail is 1 lower, and
  an action sending the 2.5 s group timer); a white mage synced to 80 (real 86) got 60 s for Divine Benison and 120 s
  for Tetragrammaton (twice the recast, though their second-charge traits are level 88 and 98), 60 s for Swiftcast
  (its −20 s trait is level 94) and the plain recast for the rest (Chronofoil capture 0.1). Before 2026-10-25 Kawari
  sent Recast100ms × the row's MaxCharges (30 s for Divine Benison) and didn't apply traits.
* **Refused actions** (ActorControlSelf **700**, `ActionRejected`: the LogMessage, the request's action type, the
  action, and a fourth value): the client prints the message and drops the request. In Bob's Chronofoil captures (42
  sessions, 2026-06 to 2026-10) retail refused 18 requests: 563 "Invalid target." nine times, eight of them for a
  target killed 0–0.2 s before the request arrived (the ninth had gone); 579 "Cannot execute at this time." twice
  (once the caster had just died, once something else changed their Condition); 582 "Not yet ready." three times
  for a Glare III sent during the previous GCD's cast, with the recast group's elapsed time in hundredths of a second
  as the fourth value (120, 151, 191); 1952 four times for an item used where it can't be. Kawari answers the first
  two cases the same way (`rejection`, since 2026-10-25): a player's action while dead (579), and one aimed at a
  character with no health left, unless the action may be used on the dead (563; Action.DeadTargetBehaviour, Raise).
  Retail never refused for range, facing or line of sight, so Kawari doesn't either. What the client checks itself
  before sending (ActionManager.GetActionInRangeOrLoS and GetActionRange, 2026.10.16 client):
  * ground distance (x and z) at most Range + the caster's hitbox radius + the target's (in PvP, against a player, the
    caster's radius isn't added), and a height difference at most the larger of Range and the target's radius
    (except in zones with a flag the client reads), else 566 "Target is not in range.";
  * Range is the Action row's; −1 is the weapon's reach (melee and ranged auto-attack distance), and Embrace, Fey
    Union and Seraphic Veil (802, 7438, 16548) reach 30 yalms; under Briny Veil (status 433) it's cut by Range ×
    stacks / 16;
  * NeedToFaceTarget: the target within 45° either side of where the caster faces (unless closer than 0.06 yalms),
    else 565; RequiresLineOfSight: a ray from 2 yalms up on the caster to 2 yalms up on the target, else 562 "Target
    not in line of sight." Neither when the target is the caster.
* **MP costs** as the client works them out (ActionManager.GetActionCost, 2026.10.16 client): PrimaryCostValue ×
  ParamGrow.MpModifier of the caster's level / 100, and MpModifier is 10000 at every level, so the sheet's value × 100
  at every level, as Kawari charges. The client's own exceptions: Cure II costs nothing under Freecure (155) and Fire
  III under Firestarter (165), both done in the scripts; Adloquium, Succor, Indomitable, Excogitation and Concitation
  under Recitation (1896); Black Mage's Astral Fire and Umbral Ice change fire and ice spells' costs (×2, or nothing).
* **Auto-attacks** come every weapon delay × haste / 100 (2 s without a weapon); they were every 2 s for everyone.
  Retail: Paladin 2.27 s for a 2.24 weapon, Warrior 3.38–3.40 for 3.36, Monk 2.05 (2.56 × 0.8 with Greased Lightning),
  Bob's 2.41 for 2.4. As on retail (Bob's Chronofoil captures, 663 of his auto-attacks), a swing (`auto_attack_target`)
  goes to the player's **current target**, not the one auto-attack was turned on with (after a target switch the next
  swing went to the new target, never the old one, and the client doesn't re-send ToggleAutoAttack), only to a living
  enemy (or duel opponent) within **3.5 yalms of its hitbox** (3 + the player's 0.5, on the ground; Archers, Bards,
  Machinists and Dancers 25, Sapphire's), and never while casting (2 of 663 during a cast). A Gladiator's 597 all came
  within 3.6 yalms of the hitbox, and none in 17 s spent 3.6–4.7 off. A swing that can't be made stays ready and comes
  as soon as it can (retail: 0.1 s after stepping back in). Auto-attack stays on meanwhile; the client turns it off a
  median 0.05 s after its target's HP reaches 0. Kawari used to keep hitting the toggled target from any distance
  (9–21 yalms in the 2026-10-20 bundle). Each swing shows the weapon's damage type (retail 0x71 slashing for
  Gladiators, 0x73 blunt for Conjurers/White Mages; Kawari sent 7 "Physical") and one of three swing animations
  (ActionEffect byte 30, `animation_variation`, 0–2 at random), and names its target with object type 0. Its damage
  was always applied (HP, enmity, UpdateHpMpTp, sent just before the ActionEffect); the auto-attack to Fast Blade ratio
  is retail's (level 15 Gladiator: 13 / 36 on retail, 10 / 27 on Kawari).
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
  `players_by_level` (`{"<level>": avg}`) is for a kind met at several levels (2026-10-27): the tainted earth sprite
  (BNpcBase 294) hit for 7 at level 5 in Trial by Earth and 21 at level 20 in Sylphie's Trials; the NPC's own level's
  comes first, then `players`, then the nearest level's (`Timeline::recorded_amount_at`).
  `by_distance` (`[[yalms, amount], ...]`, 2026-11-06) is for an attack weaker the further from the caster it lands:
  what it did to players at those distances from the caster's middle, straight between two points and the end's
  beyond them; every hit (player or NPC) is its recorded amount times that over `players`
  (`RecordedAmount::distance_factor`). The Navel's Geocrush: 991 at 4.7 yalms, 367 at 22 (17 hits in Bob's three runs
  on a line, 1161 less 36.1 a yalm).
* **Party bots and party NPCs are hit as players are** (since 2026-10-23): an enemy's hit on one is scaled as a hit on
  a player (its `players` recorded amount, or `npc_damage_scale`), with a player's defences for a bot
  (`modifiers::apply_hit_on_player`) and, for a [Party NPC](party-npcs.md#taking-hits), its member's share of a
  player's damage, or its timeline's `npcs` amount where that was recorded on the Duty Support members. Before, they
  were hit like any NPC by another, unscaled: about eight times a player's damage, and Sastasha's Duty Support members
  died to two enemies (Bob, 2026-10-23).
* An NPC hitting another NPC with an action whose timeline records only a `players` amount hits it for that amount
  too (since 2026-11-10; across every timeline with both, the median `npcs`/`players` ratio is 0.98). Before, those
  hits went unscaled: quest battle 60's wyvern auto-attacked Haurchefant for 229 where it hit players for 96, and the
  second wave's archer for 240 against 85 (Bob's bundle 20261110-095652).
* Other NPCs hitting other NPCs without a recorded amount at all are not scaled.
* An NPC's **damage over time** is scaled as its hit was (`scale_npc_periodic_damage`): the action script works the
  tick out from the NPC's stats as if it were a player's, which made Victory in Peril's corsair conjurers' Aero tick
  for 26–28 where retail's were 2–7 (the hit, scaled to 7, was right). Aero's tick is 30 potency to the hit's 50, so
  the same scale gives about 4.
* An NPC action with **no hit, only a damage over time** (the Thousand Maws of Toto-Rak's poisons) has its recorded
  amount as the tick (`set_npc_periodic_damage`): retail's ActionEffect gives the tick with the status (effect kind 14:
  the status, the tick, 50), 57 on the player and 19 on a Duty Support member for the room bosses' 1014.
* **Knockbacks** are an effect of their own (`TargetEffectKind::Knockback`, kind 31, since 2026-11-04): `1F 00 00 00
  00 00` then the Knockback sheet's row as a u16. The server only names the row; the client moves the player itself and
  sends its own positions as it goes. An action script adds one with `effects:knockback(row)` after the damage; a miss
  drops it (`roll_npc_hit`), and players caught by an area attack get it as the main target does. From The Lominsan
  Way's Rive (1299) on the player: `03 00 71 00 00 00 3c 00`, `1f 00 00 00 00 00 01 00`, `1b 00 00 00 00 80 13 05`
  (Bob's Chronofoil capture 44df02fc), the player then about 8 yalms further on along the line; its miss had only the
  Miss. The last entry (kind 27, 0x80, the action's ID) isn't sent; it isn't the knockback.
  Silkscreen (701) knocks back the same way (row 1 in every hit: the Thousand Maws of Toto-Rak's boss's three, Seer
  Folly's quest battle boss's seven, its targets 9-10 yalms further off; `TotoRakFinalBoss_00701`). The kind-27 entry
  (`1B`, the action's ID, 0x80) came with every action that hit in both captures.
* Drains (`heal_self` effect, e.g. Blood Drain) heal the attacker by what they took; the attacker's HP is sent after.

A timeline with `timeline_always_plays` (only Sastasha's giant clams have one) runs whether or not the NPC is
fighting; its actions still need a target. Nothing read the flag before, so the clams never opened (Bob's bundle of
2026-10-23). The clams' loop is retail's: shut for 4 s, open for 11 s (474 changes of their shells' state in Bob's
capture of five runs; Kawari's whole-second timelines make it 16 s, shut 5); they never attack.

Timeline fields the fork uses: `from_pull` (times from the NPC's pull to the half second instead of whole seconds from
its spawn; each point fires when due, in reach or not, and may repeat every `every` seconds; an action point's
`target` is `target` (default), `other` (someone else on its enmity list at random), `itself` or `nobody`, and one on
`itself` goes the way of `facing`: `target` (default, the one it's fighting) or `player` (the nearest player, whoever
it's fighting: The Lominsan Way's Rive); a point with `hp_below` comes once as its health falls to that percentage instead of at a time, one with `restarts` starts the
clock again as it's used, and one `after` an action counts from that action's last use and comes only while it's the
last to restart the clock: Kottos and the Copperbell Mines' other two bosses; the director can start a phase itself,
`restart_timeline`, as Ifrit's),
`hitbox_radius` (reach includes it), `autoattack_action_id` (870/871/872 per kind),
`attack_range` (default 5 yalms; Papalymo 10), `damage_to_players` (`{"level": damage}`), `recorded_amounts` (above),
and `timepoints` (the
skills an enemy uses in turn). An action timepoint can carry `lockon` (a Lockon row): the marker goes over the target as
the cast begins (ActorControl 34 from the target, `ShowLockonVfx`), as retail marked the Copperbell bosses' tank-busters
(218 on 28545, 28461 and 28764, every cast in Bob's three runs). An NPC that dies mid-cast has its cast cut off for
everyone (ActorControl 15 with 540, 1, the action, 1; `actor::kill_actor`), as retail. A trailing no-op timepoint keeps the real ones evenly spaced because the loop hits its
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
| Western Thanalan (140) | `wil_w1f1_planmap.json`, layer `BNPCS_RETAIL` (397) | Bob's Chronofoil capture of 2026-10-18 (walking the zone) with the older captures: `questkit chronofoil --dropin 140 --appends bg/ffxiv/wil_w1/fld/w1f1/level/planmap.lgb`; 16 more from his capture of 2026-10-19 |
| Eastern Thanalan (145) | `wil_w1f3_planmap.json`, layer `BNPCS_RETAIL` (370), Unsolved Mystery's `QB_SubWil110_001` and For the Children's `QB_SubWil113_001` (2 each, `nonpop`, display flags 0x40003; from Bob's Chronofoil capture b8432613), and quest battle 45's `QB_ManFst209_001` (territory 275, a copy: 26 BNpcs and the scout, an event NPC, all `nonpop`; the same capture) | Bob's Chronofoil capture of 2026-10-19 with all the older ones: `--dropin 145 --appends bg/ffxiv/wil_w1/fld/w1f3/level/planmap.lgb` |
| Southern Thanalan (146) | `wil_w1f4_planmap.json`, layer `BNPCS_RETAIL` (448) | the same: `--dropin 146 --appends bg/ffxiv/wil_w1/fld/w1f4/level/planmap.lgb` |
| Northern Thanalan (147) | `wil_w1f5_planmap.json`, layer `BNPCS_RETAIL` (99) | the same: `--dropin 147 --appends bg/ffxiv/wil_w1/fld/w1f5/level/planmap.lgb` (a short visit: 21 kinds) |
| Mor Dhona (156) | `lak_l1f1_planmap.json`, layer `BNPCS_RETAIL` (173) | Bob's Chronofoil capture of 2026-10-25 (flying around Mor Dhona and the Shrouds) with all the older ones: `--dropin 156 --appends bg/ffxiv/lak_l1/fld/l1f1/level/planmap.lgb --exclude-base 3208` (the S rank), plus six spots only the older captures had. Until then Mor Dhona's spawns were in `roc_r1f1_planmap.json`, appended to Coerthas Central Highlands' LGB (`roc_r1/fld/r1f1` is territory 155; Mor Dhona's Bg is `lak_l1/fld/l1f1`): Mor Dhona had none and Coerthas had Mor Dhona's |
| Coerthas Central Highlands (155) | `roc_r1f1_planmap.json`, layer `BNPCS_RETAIL` (378) | Bob's Chronofoil capture of 2026-10-25 (half an hour flying around it) with all the older ones: `--dropin 155 --appends bg/ffxiv/roc_r1/fld/r1f1/level/planmap.lgb` |
| East Shroud (152) | `fst_f1f2_planmap.json` (450) | the same capture with the older ones: `--dropin 152 --appends bg/ffxiv/fst_f1/fld/f1f2/level/planmap.lgb` |
| South Shroud (153) | `fst_f1f3_planmap.json` (397) | the same, `f1f3` |
| North Shroud (154) | `fst_f1f4_planmap.json` (445), and Trial by Earth's `QB_ClsCnj001_001` (1, `nonpop`, display flags 0x40003) | the same, `f1f4`; the quest's from Bob's Chronofoil capture b23b91ef |

Every zone's drop-in was checked against all 64 of Bob's captures (2026-10-25): its `appends` is the territory's own
`Bg`, and every spawn point in its retail layer was seen in that territory; the only spots no capture shows are the
quest battles' `QB_*` layers (spawned by their quests) and upstream's Central Shroud layer, `LVD_kakashi`.
| Middle La Noscea (134) | `sea_s1f1_planmap.json`, layer `BNPCS_RETAIL` (376), and quest battle 35's `QB_ManSea005_001` (8, `nonpop`) | Bob's Chronofoil capture of 2026-10-21 (flying through every La Noscea zone, an hour) with all the older ones: `--dropin 134 --appends bg/ffxiv/sea_s1/fld/s1f1/level/planmap.lgb` (with upstream's gathering points, `s1f1_planlive.json`) |
| Lower La Noscea (135) | `sea_s1f2_planmap.json` (400) | the same, `s1f2` |
| Eastern La Noscea (137) | `sea_s1f3_planmap.json` (241) | the same, `s1f3` |
| Western La Noscea (138) | `sea_s1f4_planmap.json` (294) | the same, `s1f4` |
| Upper La Noscea (139) | `sea_s1f5_planmap.json` (137) | the same, `s1f5` |
| Outer La Noscea (180) | `sea_s1f6_planmap.json` (167) | the same, `s1f6` |

Central Thanalan's spawn points carry retail's level and HP for each (levels 1–17, plus four level 35 Ul'dah guards at
their posts and one level 50 hunt mark, Ovjang, kept to one of its spots), whether it's aggressive (97 of 632) and its
links. The 21 FATE monsters seen were left out; their FATEs spawn them. Timelines (hitbox, auto-attack, damage to
players) were added for the 17 kinds that had none, from the Recorder export of the same day: every hitbox is
retail's; the auto-attack is the one seen in the captures for the Sun Bat, Antling Soldier, Snapping Shrew, Star
Marmot, Huge Hornet and Giant Tortoise (the rest keep the default 870); damage to players for the Snapping Shrew,
Star Marmot, Huge Hornet and Giant Tortoise; the Star Marmot also uses Somersault Slash.

Bob's Chronofoil capture and Recorder export of 2026-10-18 (a gladiator, levels 6–12, through Central and Western
Thanalan) added: the auto-attack of the Toxic Toad (872), Syrphid Cloud (872), Quiveron Guard (872), Quiveron
Attendant (873) and Hammer Beak (871), which had the default 870; damage to players for the Toxic Toad, Syrphid
Cloud, Orobon, Quiveron Guard, Quiveron Attendant, Hammer Beak and Antling Soldier (the medians of the hits on Bob,
a tank, so probably low for other roles; the kinds that already had numbers kept them); the Orobon's Gnaw (541),
once early in each fight; and drops for the Star Marmot (13 of 23 kills: Garland's 5291, two at a time, or 4758) and
the Huge Hornet (2 of 13: 4850) in `battle/Drops.lua`, whose entries can now say how many. Skills seen but not
scripted: the Toxic Toad's Sticky Tongue (347) and Labored Leap (1006), the Hammer Beak's Poison Breath (507), the
Qiqirn Scrambler's Faze (495), the Black Eft's Stagnant Spray (448), and the Quiveron Attendants' archer kit
(Hawk's Eye 99 first, then Raging Strikes, Heavy Shot, Venomous Bite) and the Guards' Featherfoot, Footwork and
Triple Threat (55, 476, 475).

Western Thanalan's 381 spawn points (42 kinds, levels 1–24, plus the level 35 Flame and Brass Blade guards and the
level 48–49 4th Cohort near its edge, as retail had them; 74 aggressive) were made the same way, leaving out 12 FATE
monsters and 5 that belonged to someone. Its quest battle copy (273) loads the same drop-in, but quest battle
instances drop the open-world NPCs. Timelines for the 31 kinds that had none (and Central Thanalan's Stone Torch)
come from the Recorder export: hitboxes, the auto-attack used most (the Sun Midge Swarm's 872, the 4th Cohort
Eques' 871, the rest 870) and the damage to players recorded for the Yarzon Scavenger, Sun Midge Swarm, Desert
Peiste, Rusty/Copper Coblyns (base 187) and the Eques. Their skills (Foul Bite, Sanguine Bite, Body Slam, Cold
Gaze, Shatter, the gigantoad's Sticky Tongue and Labored Leap, the guards' Rampart, Fight or Flight and Fast Blade)
aren't scripted. The Sun Midge Swarm drops 5492 (2 of 4 kills).

Eastern, Southern and Northern Thanalan and Mor Dhona came from Bob's Chronofoil capture of 2026-10-19 (flying
through each zone) with every older capture: retail's levels (Eastern 5–44, Southern 25–49, Northern 49–50, Mor Dhona
44–50), HP, aggression, links, positions and facings. Left out, as before: FATE monsters (53) and NPCs that belonged to
someone; S-rank hunt marks (Eastern Thanalan's, base 3198, and Central's 3197), whose spawns are conditional; and every
spot but one of each A/B-rank mark (bases 3179–3196), as Central Thanalan's Ovjang. The level 35 Brass Blades and
Flames and Northern Thanalan's level 50 guards are where retail had them. None of their kinds has a timeline yet
(no Recorder export of these zones), so they fight with the defaults (auto-attack 870, a default hitbox and damage).
Central Thanalan's capture had nothing its drop-in lacked.

The six La Noscea zones came from Bob's Chronofoil capture of 2026-10-21 (2026.10.16, an hour flying through them)
with every older capture, written whole by `questkit chronofoil --dropin` (guards, Behavior and `stationary`
included): 1615 spawn points, 121 kinds, levels 1–50 (Middle and Lower 1–50 with their marks, Western 10–50, Upper
20–50, Eastern and Outer 30–50), 584 aggressive, 122 stationary, and Limsa Lominsa's Yellowjackets (bases 783–790,
Behavior 30051–30055, character data flags 0x4) where retail had them. Left out: 102 FATE monsters, 11 that belonged
to someone, Upper La Noscea's S-rank mark (base 3205, 249,139 HP) and all but one spot of each A/B-rank mark (bases
3184–3189, about 20,000 HP). No timelines yet (no Recorder export of these zones): they fight with the defaults. The
`Bg` folders (`sea_s1/fld/s1f1`–`s1f6` for 134, 135, 137, 138, 139, 180) are TerritoryType's, not read from the game
here: a drop-in whose LGB isn't the zone's doesn't load (no "Loaded dropin" line for it in the log).

**Several drop-ins per zone**: the zone loader took the first drop-in matching the zone and stopped, so Middle La
Noscea would have had either upstream's gathering points or its battle NPCs. It now loads every matching drop-in in the
first folder that has any (sorted by name), so an extra resource path still overrides the built-in ones.

SpawnNpc's character data flags (byte 8) aren't only "hostile": retail sends 0x1 for passive monsters, 0x2 for
aggressive ones and 0x4 for guards (Ul'dah's Brass Blades and Flames, bases 791–798 and 1342; Gridania's Wood Wailers,
580–583; Mor Dhona's 1776–1778; 22 kinds in all, Bob's Chronofoil captures). Guards also have their own Behavior row
each (30051–30055; ordinary monsters 30246). A drop-in battle NPC can set both (`character_data_flags`, `behavior`), and a mount it comes out riding (`mount`,
CommonSpawn's current_mount; `director:dismount_bnpc` gets it off, [Quest battles](quest-battles.md)).
With 0x4 the client treats the NPC as a friendly one with a talk: clicking it asks the server for its DefaultTalk.

**Guards** (all 36 in the Thanalan, Mor Dhona and Central Shroud drop-ins) are as on retail: display flags 0,
character data flags 0x4 (`CharacterDataFlag::GUARD`), their Behavior row. Bob's Chronofoil capture of 2026-10-21
(2026.10.16) has both Horizon guards, server spawns with the same flags (layout 4057563, base 795, the Lalafell by
the west gate; layout 4057553, base 798, by the east one), battalion 0 (monsters 4):

* **They defend their post.** A guard notices monsters the way aggressive monsters notice players (`can_notice`: 15
  yalms in front, 11 all round, battalions that are enemies), never players, and fights them. The Lalafell left his
  post for a monster 14.5 yalms off, drew his weapon (ActorControl 0, 1), went into combat (ActorControl 4, 1) and
  targeted it, fought it with auto-attacks (870) and weaponskills (717, 718; the game credited him as first attacker),
  took on a second one, then sheathed (ActorControl 0, 0), left combat and **walked** back, turning on the spot to his
  spawn rotation (−1.462) at the end. Kawari: an NPC at a post (a guard, or a `stationary` spot) that loses its
  target walks back home still watching (it can pick up another monster on the way; leashed enemies run home ignoring
  everyone instead), and on arrival turns to the rotation it spawned with (`home_rotation`), one ActorMove. Out in
  the world only. Someone it can't get to (no navmesh path, or a path whose end is no nearer than where it stands) it
  leaves alone for a minute (`UNREACHABLE_FOR`, `Instance::unreachable`), dropped from its enmity and not noticed
  again: the Brass Blade at Scorpion Crossing (layout 4057530, its post 0.63 yalms off the navmesh) noticed a monster
  it couldn't reach, gave up, walked home, noticed it again and stepped between its post and the mesh's edge every
  half second for five minutes (Bob's bundle of 2026-10-25). Arriving is judged from where the movement pass put it
  this tick (`npc_behavior`'s mut pass applies the const pass's ActorMoves first): judged from where it stood a tick
  before, the same guard back at his post still looked 0.63 yalms off it, planned the last step again from the mesh's
  edge and walked edge, post, edge every half second, facing −1.07 throughout (Bob's second bundle of 2026-10-25). The nameplate shows while it's fighting, so the one that saw no monster showed none.
* **Clicking one plays its line.** Retail answers the client's StartTalkEvent (DefaultTalk 1838 for base 798, 1839 for
  the Lalafell) with EventStart (Talk), then scene 0 with flags 0x2001 (HIDE_HOTBAR | NO_DEFAULT_CAMERA) and no
  parameters; the client shows the line and returns the scene (EventReturnHandler2), and the event finishes.
  Kawari's DefaultTalk handler (`DefaultTalkEventHandler`) does that; before, it sent EventStart and nothing else, and
  the client waited for the scene forever (Bob's soft lock, 2026-10-20). A DefaultTalk nested in a SwitchTalk keeps
  its own script's scene (`on_talk` only runs for the client's own talk). When the guard was busy fighting, retail
  ended the event 0.4 s in without a scene (EventFinish with 0x1D); Kawari always plays it.

(From 2026-10-20 until this, the guards were decorative on Kawari, display flags 0x208, from a misremembered retail.)

Kill EXP checked on the 105 solo kills of that export: Kawari's formula (the level's ScaledQuestXP, full down to two
levels below, then 10 % less a level) gives the recorded EXP for all but level-ups during the fight, chains and kills
that gave nothing.

## Enemy behaviour (`server/npc_behavior.rs`)

| Mechanic | How it works |
|---|---|
| **Enmity** | Each NPC keeps an enmity table. Damage adds to it (×10 in a tank stance: Iron Will, Defiance, Grit, Royal Guard; no tank bonus without one), damage over time ticks count for whoever applied them, and healing adds half of what it actually restored (overheal doesn't count) to every enemy that has the healer on its list, shared between them; the NPC attacks whoever it hates most. Directors add enmity with `set_hate`. Fitted from Bob's captures (39,248 enmity lists in 22 duties): damage ×1, stance ×10.05–10.8, healing 0.43–0.57. Retail's Provoke puts the provoker at the top plus a fixed amount (effect kind 24, 18,404–63,118 by player, not max HP or level; Kawari: top + 1); everyone in the party goes on an enemy's list at 0 when one of them engages it (not done). Hits on an NPC that can't be hurt (`currently_invulnerable`) add none either, aimed at it or caught in an area attack (which shows it effect 7 and takes nothing from it, as its target does) (since 2026-11-10: quest battle 60's wyvern turned on Haurchefant, who kept hitting it, where retail's stayed on the player in every run). A heal on the target, or an action on oneself, adds no enmity to the target (healed enemies used to put their healer on their list and turn on it once their own target fell, QB 38). |
| **Enmity UI** | The HaterList (every enemy with the player on its list, the player's enmity as a percentage of that enemy's top, floor) and the EnmityList (the player's target's list, highest first, the top at 100, at most 8; only the player's party on it, players, party bots and Duty Support's members, the top among them at 100: since 2026-11-10, retail's quest battle 61 listed the player alone at 100 while the boss was on the gladiator ally, five runs, and Kawari listed the allies too) are sent only when they change, and an empty HaterList once as the fight ends, as retail. They went out every half second to everyone, even out of combat, with everything at 100. (The lists' entries start right after the count, as Kawari's structs already had them.) "Every enemy with the player on its list" means its enmity list, not whom it's going for: the haters were counted from each enemy's target, so an enemy left the player's HaterList (count 0) the moment the tank took it and showed 100 % while it was there; retail kept it at the player's 0-1 % while the tank held it (Bob's Copperbell Mines capture with Duty Support, `bd298651`, and bundle of 2026-10-25). The list keeps the order it was last sent in, newcomers after, as retail's did. |
| **Auto-attacks** | Every 3 s (retail 3.03 s over 8,200; a few kinds 4 or 5 s), timed per NPC from when it picked up its target. With a cast time they show a cast bar. |
| **Cast bars** | An NPC's ActorCast shows the action's cast time minus 0.3 s, and the action still lands at the full time: retail's for 99 % of 8,392 enemy casts (`npc_cast_bar_time`). |
| **Speed** | Running (chasing, going home, following) 6 yalms a second, walking about while idle and on a patrol path 2 (retail's ActorMove speed byte, tenths of a yalm a second: 60 and 20; a few kinds chase at 3.) It was 4 for everything, always with the running animation. Players walk at 24 and run at 60 (Sprint 78, Peloton 72, ground mounts 90–150). |
| **Moving** | An NPC is only sent an ActorMove when it moved or turned. Sending one every half second to where it already stood (as reaching its goal and facing its target did) made the client play a step each time: idle NPCs flickered between walking and standing, and out in the world each sent about 1400 a session for a handful of real moves. In content an NPC with no path at all to its target (e.g. starting off the navmesh) walks straight at it instead of dropping it; dropping it sent To Guard a Guardian's second wave wandering back around the tree. |
| **Stance** | NPCs draw their weapon when they pick up a target, once, and sheathe 2 s after losing their target, as retail (94 ToggleWeapon in the Copperbell Mines' first stretch, about one per pull). Until 2026-10-25 they drew it again 0.5 s after every action (523 in the same stretch), as NPCs had seemed to drop out of their stance after acting (2026-10-13); their ActionEffects then carried a player's lock (0.6) and swing 0, retail's NPC lock and swing now (below), which is the likeliest cause. If NPCs are seen sheathing between attacks again, that's where to look. |
| **NPC ActionEffect** | Auto-attacks (the timeline's `autoattack_action_id`, 870, 872…) lock for 0.1 s and swing one of three ways at random (0, 1, 2), as retail's 547 in the Copperbell Mines; their other actions carry the lock retail sent for them, the same every use (`Timeline::animation_locks`: 2.1 for most bosses' attacks, 2.3–2.4 for some trash abilities, 1.1 for helpers' markers, 0.1 for the gigas casters' 968), else a player's (0.6, or 0.1 after a cast). 870 is slashing (0x71: every 870 hit there, six kinds of enemy), 872 blunt; 870 was a link to 872's script. |
| **A player's area heals and buffs** | Medica, Medica II, Succor and the like land on the caster and their party in range: players, and party bots and Duty Support's members (since 2026-11-10; only players counted, so Bob's Medica in the Stone Vigil healed him alone with his three party NPCs 1-13 yalms off, bundle 20261110-134029). |
| **Area results** | Everyone an area action lands on is shown in one packet with what it did to each: AoeEffect8 (636 bytes: ActionEffect's header, 64 bytes of effects per target slot at 42, the targets at 560, the packed aimed-at point at 624), as retail sends area actions that caught no one, several, or were aimed at the ground or at no one (animation target 0xE0000000 and the point then); one aimed at someone that caught one goes as ActionEffect1 (Copperbell Mines: 838 ActionEffect1 with one target, 172 AoeEffect8, 164 with none). Players' area actions (Medica on four) and NPCs' area attacks alike; each target shown gets its EffectResultBasic as the hit lands. Kawari used to show one target and send everyone else flying text (ActorControl 1541, which retail uses for damage over time ticks); only those past the eighth still get it. An NPC's area attack cast on itself shows the caster as the animation target. |
| **Health updates** | UpdateHpMpTp only when the numbers changed since the last one sent (`Instance::last_hp_mp`), and never for an enemy: retail's enemies' health reaches clients with the hits that change it (EffectResultBasic) and their StatusEffectList otherwise (a damage over time tick: ActorControl 1541, then the list carrying the HP). In the Copperbell Mines' first stretch retail sent 562 UpdateHpMpTp (the player's and the party NPCs'), Kawari 2,295, most repeating the last (the tank bot every half second). |
| **NPC statuses** | An NPC that gains a status from an action (its target, or the caster on itself) has its StatusEffectList sent as well as the EffectResult, as retail does on every gain and loss for party NPCs and enemies (618 lists from NPCs in the Duty Support run, 15 in Kawari's); losses already sent it. Without it a damage over time on an enemy or a party NPC's buff didn't show on its nameplate or in the party list. |
| **Casts and auto-attacks** | An NPC doesn't auto-attack in the tick it begins a cast (the auto-attack's own cast record replaced the cast's, so it went on walking, turning and swinging through the cast bar and its area landed facing wherever it had turned, not where the client drew it: Kottos, Bob's bundle of 2026-10-26). |
| **Placement** (`server/placement.rs`) | An NPC given `keep_near` (anchors and a distance, by the director, or a party bot's own: a ranged bot within 20 of the one it follows, the tank within 15 of the player) heads back towards the middle of its anchors whenever it's further than that from any of them, whatever it's fighting, until it's 3 yalms inside the distance (`RETURN_MARGIN`), and attacks again from there; and a chase after its target stops 3 yalms inside it too (`Placement::chase_goal`). A tank's spot across its target from the party (`opposite`) is taken only when it's that far inside its anchors' distance too: otherwise it went round, strayed, came back, and went round again every second, through its target (Kottos, 2026-10-26). Sent back to the middle and let chase as far as it liked, a ranged party NPC ran up, attacked, walked back two steps and ran up again, back and forth (Bob's Copperbell Mines bundles, 2026-10-25). One given `stand_behind` (a tank and a distance) goes to the spot that far beyond the enemy the tank is fighting, on the far side from the tank (out of the enemy's frontal cones), when that spot keeps it near its anchors and its own target is in reach from there; within 2.5 yalms counts as there. |
| **Pathing** | A new path starts from where the NPC actually is, and changing target drops the old path (no more 14–16 yalm jumps). When the navmesh doesn't reach the target, Detour returns a *partial* path (status `0x40000040`, success + partial, which used to be logged as an "Error in dtNavMeshQuery_findPath"): the NPC walks it and then straight on to the target, if that last stretch is at most 15 yalms (`MAX_OFF_MESH_GAP`), and in content however far. Before, it stopped at the edge for good, out of range (To Guard a Guardian's allies stuck 9 yalms from two Ixali, so the battle never ended, 2026-10-16 bundle). |
| **Walking on the collision** | Every step an NPC takes (enemies, bots, pets, Party NPCs, wandering and chasing alike) goes over the zone's collision as a player's would (`collision.rs`, `Collision::walk`): the floor under it (not the straight line between two corners of its path, which went through hills and over dips), up steps no higher than RecastNavimesh's AgentMaxClimb, stopped by a wall (rays at a body's height over the step, its AgentRadius ahead) and sliding along it, and down off a ledge onto the floor below. Open-world enemies step down at most 3 yalms (`OPEN_WORLD_MAX_DROP`, a guess: retail's keep to their navmesh and don't throw themselves off cliffs after you); bots, pets and everyone in duties step down as far as their path goes down and 3 yalms more (`path_max_drop`, `PATH_DROP_MARGIN`; since 2026-11-02: before, as far as a player can, so in Haukke Manor the tank bot and three enemies cutting a corner of their path stepped into a sunken space with no railing in Kawari's collision and on down to the cellar, 18.8 yalms under the player, where no one went in four retail runs). A step cut short leaves the NPC where it stopped, and its path goes on from there; one held up by a wall (or a drop) for 2 s (`NPC_WALL_PATIENCE_TICKS`, as party bots' `WALL_PATIENCE`) walks on along its path, and the server logs where; never through a gimmick the director put up (a sealed room's wall, a door it shows as locked: `Walk::sealed`, since 2026-11-02, as for bots), and since Bob's Haukke Manor bundle 20261103-051833 not past one either (held up by the wall beside a shut door, it went on through the door: `Collision::crosses_sealed`), and only onto a floor no deeper than it would step down (`past_a_wall`: put on its path's height over a hole, it sank to the floor below the next tick, and bots and enemies walked on past a shelf by Haukke Manor's sunken space and on down to the cellar). Since 2026-11-01 that's every NPC, open-world enemies too: retail's server walks them on its navmesh alone, and a Ziz in the North Shroud stood 8.5 yalms short of the player it was after, never attacking, for as long as he watched (Bob's bundle 20261101-215226); out there a path keeps to the navmesh but for at most its last 15 yalms (`MAX_OFF_MESH_GAP`) (Yda stood 16 yalms from Brotherly Love's beast for the rest of the fight: her path ended where the navmesh did, the rest went straight, and a wall stopped every step; with the path never used up she never planned another. Bob's bundles 20261030-110117 and -110334). The collision is every mesh the navmesh is built from, kept as triangles in a 2-yalm grid, and since 2026-11-02 (collision version 4) the simple shapes too, and (version 5) a door standing open without its closed door (a door's shared group placed `Open` leaves out its collision boxes: Haukke Manor's open doors were shut in Kawari's collision and navmesh, a box 4.6 yalms across and 10.5 high in each doorway, so bots stopped at them and paths through them went nowhere): a BgPart whose collision is an analytic collider (a box, cylinder, sphere or plane: its bounds give its size and middle, then its own transform, then the BgPart's, as vnavmesh places them; 569 of Haukke Manor's 1199 BgParts with collision, among them the invisible walls by its sunken space) and a collision box of those shapes (before, boxes only; every zone's navmesh and collision are made again the first time it's entered): `kawari-navimesh` writes it beside the navmesh (`<navmesh>.col`), and an instance whose navmesh has none has both made again in the background (one generator per zone, however many instances), keeping to the navmesh's floor until then. Since 2026-11-10 the navmesh uses Detour's 64-bit polygon references (recastnavigation-sys' `detour_large_nav_meshes`; the file starts with `KNV64`, collision version 6): with 32-bit ones a navmesh holds at most 2^22 tiles times the most polygons in a tile (each rounded up to a power of two), and Coerthas Central Highlands' went over it (most likely since it was made again with the simple shapes, 2026-11-02), so Detour wouldn't take it and the world server panicked as Bob logged in there (his log of 2026-11-10). A navmesh Detour won't take is now left out with a warning saying why (no paths there) instead of a panic, and `kawari-navimesh` logs an error when the one it made won't be taken. Older navmesh files aren't read, so every zone's navmesh and collision are made again on its first entry (until then NPCs there have no navmesh paths). From Bob's Copperbell Mines bundle of 2026-10-26 (bots through the air off a ledge, and up its cliff). The shared groups an event object controls (barriers, doors, boss-room walls) are gimmicks in it: left out of the navmesh, and gone from an instance's collision while their object is hidden there ([Party bots: gimmicks](party-bots.md)). |
| **Line of sight** | An NPC (enemy, bot, pet) acts on whom it's after only where it sees them, on the zone's collision (`Collision::sees`, `npc_behavior::in_sight`): a straight line between the two at 1.5 yalms over their feet (`LINE_OF_SIGHT_HEIGHT`, a guess) that no wall, floor or ceiling of the collision crosses, nor a gimmick the director put up (one it hasn't touched doesn't count: some aren't solid for players). Out of sight it isn't in reach: it walks on towards them along its path until it sees them (a caster comes round the corner after a player who broke its line, as retail's do), and doesn't auto-attack, cast or use its timeline's actions on them. Party bots' own choices too: a heal on someone it can't see, the tank's ranged pull, a Party NPC's kit (provoke, attacks) wait until it does. Areas need no sight (they reach 5 yalms up and down, `AREA_HALF_HEIGHT`). Without a collision (not made yet) everyone sees everyone. From Bob's Haukke Manor bundles 20261103-051638 and -051833 (bots and enemies attacking through walls). |
| **Facing** | An NPC's attack around or in front of itself (a timeline's `itself`) faces its target as it begins, and every NPC hit goes out facing the way its cast was aimed (`NpcCast::rotation`): it doesn't turn for 0.6 s after a cast ends (`KEEP_CAST_FACING`; an auto-attack's swing doesn't hold it), and the ActionEffect or AoeEffect carries the cast's facing. From Kottos' Colossal Slam and Catapult (Bob's bundle of 2026-10-26, [Dungeons](dungeons.md#the-copperbell-mines)). An NPC's cast keeps its facing to the end even if another action begins the tick it goes off (a timeline point due then; its record replaced the cast's, and the hit went the new action's way while the omen had shown the other: The Lominsan Way's Rive, Bob's bundle 20261104-190047): the hit takes the cast's request's facing then (`own_cast` in `action.rs`). |
| **Leaps** | An NPC action in `NPC_LEAPS` lands its user beside its target when retail's did (`SnapNextMove`, then an ActorMove): the Duty Support Marauder's 29291, 0.58 s after the hit ([Party NPCs](party-npcs.md)). |
| **Path ends off the navmesh** | Someone by a wall or at a cliff's edge is off the navmesh (it's kept an NPC's width from both), and a path from or to them failed outright when no navmesh was within 2 yalms ("start poly: 0" / "end poly: 0", 40 in a minute of that bundle), so the walker went straight there. A path's ends now look for the navmesh within 2, then 6, then 15 yalms (`PATH_END_SEARCH`). |
| **Wandering** | Idle open-world NPCs walk to a random navmesh spot within 5 yalms of home after a pause of 20–30 s (`MOB_WANDER_TIME` + up to `MOB_WANDER_JITTER`; before, every 10 s). Measured on retail (Western, Central, Eastern, Southern and Northern Thanalan and Mor Dhona, Bob's Chronofoil captures of 2026-10-18 and 19 with the older ones): of 1862 monsters seen idle for a minute or more, 1336 wandered, pausing a median 24 s (10–90 %: 15–29 s) and straying a median 6 yalms (90 % within 9) from where they were first seen, hostile or not; the rest stood still. Some kinds never move (the Brass Blade and Flame guards, bases 791–798; Mor Dhona's 1776–1778; a dozen others), others stand at some spots and wander at others, so a drop-in battle NPC can say `"stationary": true` (`Zone::stationary_npcs`): 257 spots in the six zones' drop-ins, the ones seen idle a minute or more without moving, and the unseen spots of kinds that never moved in two or more sightings. Sightings where it fought don't count (guards attack monsters that stray near them: the Brass Blade by Horizon's gate, layout 4057563, only ever moved fighting). Spots seen wandering, or never seen, wander. Arriving ends the walk: within half a yalm of the spot (`ARRIVE_DISTANCE`), or once the path there is only where it stands. A spot off the navmesh (spawn points by a wall, where the mesh is eroded) gives a path that starts at the nearest navmesh point and comes back, so a walk that only ended on an empty path went out and back every half second forever (56 of 69 moving NPCs in Bob's 2026-10-20 bundle, after a minute or two of wandering); spots `find_wander_position` cuts short are put on the nearest navmesh point. Checked with Kawari's own NPC behaviour on a Recast test mesh for ten simulated minutes: three NPCs (one with its home just off the mesh) walk 65–101 ticks of 1200, no back-and-forth, within 5 yalms of home; the build before looped on 993–1156 ticks. (Before that, the walk only ended Detour's path to where an NPC already is is that one point, so before 2026-10-19 the spot was kept, re-planned forever, and every NPC wandered once and then stood still, walking on the spot after a chase). Detour can give a point anywhere on a polygon touching the circle (open ground has big ones: 17 yalms on a flat test mesh), so `find_wander_position` tries four times for one within 5 yalms and otherwise cuts the way to the last short. A step that goes nowhere keeps the NPC's facing (it used to turn north at the start and end of every walk), and nothing is sent until it has moved: a walk's first ActorMove is a step, as retail's is (one that only turned it where it stood, and one at the end on the spot it had reached, made the client play a step on the spot: the looping walk of Bob's 2026-10-20 bundle), and an NPC standing in range of who it's after drops its path, so it sets off from where it stands instead of jumping back and forth along the old one. A walk to a spot also ends when the new path's end is no nearer the spot (0.1 yalms) than where the NPC stands, which is as near as the navmesh goes: by a spot off the mesh each of two points' paths ended at the other one, and a level 6 pugil in Middle La Noscea (layout 3929730) stepped between them, 0.75 yalms apart, every half second for 16 minutes (Bob's 2026-10-21 bundle). An NPC whose home isn't on the navmesh, or that can't path to its spot, stands still; the world server logs the first five of those per zone (`NPC <layout> can't wander from ...`). Without a navmesh for the zone no NPC moves or fights at all (`Navimesh path is not set` / `Failed to read ...nvm` at load). Duties' NPCs never wander. |
| **Aggro** | Aggressive enemies notice players within 11 yalms all round (`AGGRO_PROXIMITY`) or within 15 yalms in a ±45° cone in front of them (`AGGRO_RANGE`), while idle *or* wandering, and ignore players 9+ levels above them (`AGGRO_LEVEL_GAP`). From Bob's captures: 98 enemies came at him on their own from 2–18 yalms (median 9); the far ones (12–18) all faced him, every one from behind was within about 11; they still did at 7–8 levels below him (8 of 41 close passes), never at 9+ (0 of 154). Only whom they can see (`npc_behavior::in_sight`, the instance's collision at eye height): not through a wall, a floor, or a door the director keeps shut (since 2026-11-07: the Stone Vigil's 1466 behind the shut way on from its second boss's room noticed Bob at the door, 11 yalms off, and the bots spent the boss fight trying to reach it, his bundle 20261107-221820). (Retail's assist, idle enemies of the same kind within about 4–14 yalms joining when one is hit, isn't done.) |
| **Leashing** | Chased more than 40 yalms from spawn (`LEASH_DISTANCE`; retail gave up at 34–45, median 39, after 7–20 s, home 7–9 s later at a run), or once the one it's after is that far from its spawn and it hasn't reached them for 7 s (`GIVE_UP_UNREACHED_TICKS`: out of reach, no attack; retail's shortest chase before giving up), an enemy drops all enmity, regenerates on the way (10 % a tick; it used to heal to full at once) and walks home ignoring everyone until within 3 yalms. Enemies wander around their spawn, **only out in the world**: in content NPCs stand where they're put until something starts them (To Guard a Guardian's masked mage walked off while waiting to be hit). (The second rule is for enemies with no navmesh path to the player: they never moved, so never leashed, and kept the player in combat anywhere in the zone, 2026-10-16 bundle. Until 2026-10-22 it applied at once: a Qiqirn Eggdigger hit by Kuina from 10 yalms, 41 from its home, gave up every time it was hit and stood still, Bob's 2026-10-22 bundle.) **Only out in the world**: enemies in content (a quest battle, a duty) never leash. In To Guard a Guardian the second wave's lostwings, sent the long way round after Miraudont, leashed at full health, and back home picked up a background guard instead, which they could never kill. |
| **Claim** (`server/first_attack.rs`) | The first to attack an actor claims it: FirstAttack (374 on 7.57, 16 bytes: kind u32, 0, the claimer's ID u32, 0) goes to everyone nearby with the attacked actor as its source, just before the action's EffectResult, when it first takes damage or a miss: an enemy from a player or an allied NPC, a player from an enemy. It stays with the first attacker until they're gone (dead or left). Cleared (kind 0, no ID) for an enemy that gives up a chase and for a player whose fight is over (as InCombat goes; on retail 0 to 3 s after their enemy list empties). The claimer is also CommonSpawn's `combat_tagger_id`, so others see it when the actor spawns for them. From Bob's Chronofoil captures (2026-10-14 to 17): about 1,100 of them in the Central Shroud and Thanalan, every set one right before the target's EffectResult. Kawari never sent it before. |
| **In combat** | While any enemy has a player on its enmity list (the HaterList; since 2026-10-25 really its list, so a healer or damage dealer stays in combat while the tank holds the enemy, as on retail, where before they left combat and regenerated at the resting rate), the server sets the player's **InCombat** condition (wire bit 1, client condition 26), as retail does, and clears it when none do. The client then refuses Teleport, Return, mounting and talking to NPCs itself. Natural HP/MP regeneration waits for it to clear. Before this the server never set it, so the client let players teleport with enemies still after them. Zoning clears it (and Casting): no one arrives anywhere fighting, and the new zone's enemies set it again if any are after them. Until 2026-11-10 it came out of a lost quest battle still set, as the server's copy of a player's conditions started empty in each zone (so the tick there never saw anything to clear): out in Coerthas the client refused to change gear (or teleport, or mount), with no enemy about (Bob's bundle 20261110-095652). The server's copy now keeps what the connection told it as they zoned in. |
| **Area shapes** | One shape per area action, from the Action sheet as the client draws its marker (`ActionInfo::area_shape`), used for hitting and for the bots' dodging alike (`in_area`): CastType 2 a circle (on its target with a range, else around the caster), 5 around the caster, 7 on the ground where it was aimed, 3 and 13 cones (the angle from the omen, `gl_fan090` = 90°), 4, 8 and 12 lines (`XAxisModifier` wide), **10 donuts** (the hole from the omen, `gl_sircle_4010bf` = 10 of 40, a quarter; half when the omen doesn't say, logged), **11 crosses** (two lines through the caster, its way and across). Around the caster everything reaches from the edge of its hitbox, as the markers are drawn, but a cone's width is taken from the caster's middle all the way in: standing against a big enemy's side or back, inside its hitbox, is out of its cone (since 2026-11-10: Kawari caught everyone inside the hitbox, so the Stone Vigil's 1463 hit the player with 518 at 115-167 degrees off and Chudo-Yudo's Swinge killed them 75 degrees off at 2.7 yalms, Bob's bundle 20261110-134659; on retail nobody that far off either's facing was hit at 3.3-5.7 yalms, eleven times in Chronofoil captures 1cefedf4 and 341808b0). the hitbox is retail's where Kawari Recorder measured it (`hitbox_radius`), else **its model's** (`GameData::npc_hitbox_radius`): at startup the server tries four ways of reading it (ModelChara or ModelSkeleton's Radius, by BNpcBase's Scale or not) against the hitboxes Recorder measured, uses the one that matches most of them, and logs how many each matched; none matching most, none. Before (Bob's Copperbell Mines bundle of 2026-10-26) donuts and crosses had no shape (they hit nobody and nobody dodged them: the Thaumaturge stood in a donut unharmed), and the Copperbell bosses had no hitbox, so their circles and cones were a giant's hitbox smaller than drawn and bots stepped out to spots still inside what the player saw. |
| **Area attacks** | Cones, circles and lines from the Action sheet (`CastType`, `EffectRange`, `XAxisModifier`; cone angle from the omen, e.g. `gl_fan090` = 90°; see Area shapes). Aimed where the target was and the way the enemy faced when the cast started; the enemy stands still while casting; ground-targeted omens show where the target stood. They land only on players still inside when the cast ends, counting half a yalm past the edge (`PLAYER_HITBOX_RADIUS`, the player's hitbox; it was a whole yalm, so a player who had left a 6-yalm Terror Eye was still hit at 6.6 yalms) and no more than 5 yalms above or below its middle (`AREA_HALF_HEIGHT`, since 2026-11-02: the shapes were flat, and a 276's 29776 in Haukke Manor's cellar caught the player on the floor above, 18.8 yalms up; retail's area attacks never hit anyone more than 1.71 yalms above or below in 674 hits of Bob's captures with fresh positions, and 5 is a guess above that), each getting their own roll (damage, variance, miss/parry/crit, statuses). The ActionEffect packet still shows only the main target (only the one-target opcode is known). An NPC's area attack lands on everyone inside it its battalion is at war with, whichever side they're on (since 2026-11-10; it was its target's side: players and NPCs that aren't hostile, or the other way round. In the Eyes of Gods and Men's wyvern, battalion 5, at war with both sides, caught the boss and its adds as well as the player and the allies on retail, 1308-2642 off the boss in each of three runs; Kawari's only the player's side, Bob's bundle 20261110-102239), NPCs only when the caster's timeline has an `npcs` recorded amount for it; each one caught is rolled as its own kind of target (its recorded amount, or for a player without one the NPC's usual scale) and NPCs caught gain enmity on the attacker (the blanga's Terror Eye hit the allies by you on retail, QB 38). Before 2026-10-20 they only hit players, and only when aimed at one. **An NPC's area heal** aimed at one of its own side heals everyone of that side inside it the same way (since 2026-11-08: History Repeating's quest battle 86, whose 1070 healed three of its side at once in Bob's Chronofoil capture 195176f8; it had healed only its target): each healed for the timeline's `npcs` amount ±5 % (`is_heal_on_target`, `hit_area_bystander`). Its Regen (a heal over time) ticks only on the one it was aimed at; the others get the status alone. **Shelter**: an area attack listed in `action_areas.json`'s `sheltered_by` spares whoever a living NPC of the given BNpcBase stands in front of, within its `radius` of the line from the caster to them (since 2026-11-08: the Howling Eye's Mistral Shriek and its monoliths, see [Dungeons](dungeons.md#the-howling-eye)). |
| **Gazes** | An action whose script calls `effects:gaze()` lands only on those facing the caster, within 90° either way of where they look (`action::faces`); anyone looking away is missed (a Miss in their place, their status and damage both gone), and stays in the result as retail showed them. From quest battle 17's boss, whose paralysing 513 caught the player and Sylphie: in Bob's Chronofoil capture df6f5af9 (13 casts in three runs) every target facing it within 58° was paralysed and every one turned 94° or more away missed. The floating eye (BNpcBase 182, timeline `FloatingEye_182`) opens with the same 513 once a fight (Power of Deduction's, four retail fights: first in three, 5 s in in the other, missing or catching no one each time), then casts 970 every 3 s as its auto-attack. |
| **Walked to the caster (Seduced)** | An action whose script calls `effects:walk_to_caster(seconds, speed, pose)` walks each player it lands on towards the one who used it, whatever they do, for `seconds`: as the hit lands (`EFFECT_RESULT_AFTER`) the player is sent ForcedMoveTowards (fork opcode 132: `speed`, then the caster) and shown in `pose` to everyone near (ActorControl 82, Kawari has no name for it), and `seconds` later ForcedMoveEnd (fork opcode 166) and 82 with 0 (`QueuedTaskData::WalkToCaster`, `WalkToCasterEnd`). Their client does the walking. Lady Amandine's Beguiling Mist (28643, with its two Seduced statuses, 2266 and 227): `walk_to_caster(3.0, 4, 1132)`, as retail's packets were to every player it hit in all five runs (Bob's Chronofoil captures of Haukke Manor, 2026-11-02 and -03: about 3.5 yalms a second, its rotation fixed.) What retail's opcodes 820 (as it began, to the player), 839 and UpdateClassInfo (as it ended) do isn't known; Kawari doesn't send them. NPCs (party bots, Duty Support members) aren't walked: retail's Duty Support members weren't either. |
| **Status-only area attacks** | An NPC's area action spreads over its area when it does damage **or puts a status on its target** (`is_hostile_effect`): the same boss's 513 does no damage and caught both. Until 2026-11-01 only damage spread, so an area debuff hit its target alone. Only a status on someone of the other side counts: one an NPC puts on itself or on its own side is a buff, and a heal doesn't spread either (taken for attacks from `801f36f` to the next fix the same day, a self-buff with an area went to the caster's enemies instead of it: Like Mother, Like Daughter's 1683s and 1687 cast 416 on themselves, its sprite 313). |
| **Sides (who's an enemy)** | By **battalion**, as the Battalion sheet's IsEnemyTo says (`action::at_war`, `PLAYER_BATTALION` 0): a player's area actions land only on battle NPCs at war with their battalion, auto-attacks only on those at war with the players', an NPC's area attack catches its target's side by the same rule, the bots dodge only the casts of those at war with the players, and an enemy's on-ally moves pick its own side by it. Not the HOSTILE character flag (0x2) or the "Enemy" subkind (5): retail sends both for allies too. Ul'dah's Most Wanted's gladiator came with flags 2, subkind 5 and battalion 0, the thugs with flags 3, subkind 5 and battalion 4 (Bob's Chronofoil capture a763f0a0). Read by the flag and the subkind, the gladiator was an enemy: the player's Total Eclipse hit him four times, each hit put the player on his enmity list (he showed in the enemy list), and once the thugs were down he turned on the player, hitting for real with no swing shown, the client still drawing him as a friend (Bob's bundle 20261105-163414, 2026-11-05). |
| **Hurting one's own side** | An NPC's damage to one of its own side adds no enmity there: quest battle 17's boss drains a yodomi on its side, and the yodomi kept on the player and Sylphie (otherwise each drain would have turned the yodomi on it). "Its own side" is both the character flag's side (HOSTILE or not) and battalions not at war (the Battalion sheet's IsEnemyTo; players and their bots are battalion 0, monsters 4). From `801f36f` (2026-11-01) to 2026-11-04 it was the flag alone, so a passive monster (no HOSTILE flag, like most of the low-level ones round the cities) counted as the player's side: a player's single-target hits on it gave no enmity, so it didn't fight back and its death credited no one (no quest kill, Hunting Log entry or EXP; Bob's bundle 20261104-130013, Way of the Gladiator's kills not counted). Area hits were never affected. |
| **Ground-targeted actions** | The client's ActionRequestGroundTargeted (fork opcode, see [Fork Opcodes](opcodes.md)) is answered like an action on the player themselves, so the hotbar gets its result; placing the area (Asylum's AreaObject: base 1771, radius 15, owned by the caster) isn't done. Before, the request wasn't understood at all. |
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
| No GCD actions | A kit with nothing on the GCD (NPC actions in no recast group: Victory in Peril's thaumaturges' Fire 966, the corsairs' Fast Blade 717 and True Thrust 722, the blanga's Bloody Claw 739) uses each action as its `every`, or its recast and at least `pace`, allows. Kept to the weaving rule they never acted: there was no GCD action to weave after (Bob's bundle of 2026-10-22). |
| `refresh` | Not again on the same target for that many seconds: a damage-over-time or debuff, re-applied as it wears off (timed per target, since players' statuses aren't visible here). |
| `then` | The GCD action that comes next (a buff's follow-up). |
| `target_hp_below`, `self_hp_below` | Only below that health %. |
| `on_ally` | A BNpcBase: the action goes on a living NPC of that kind on its own side (the lowest layout ID of them) instead of its target, and waits while there's none (`Situation::allies`, `target_for`). Quest battle 17's boss drains (1127) a yodomi on its side every 26 s while one is out. |

A rotation's action replaces that tick's auto-attack; casts work as for any NPC action (it stands still while casting).
Heals on allies stay in the content's Lua (`runHeals`), since they pick a target other than the NPC's own.

## Fall damage (`server/fall.rs`)

Retail's falls (Bob's Chronofoil captures, 2026-10-28: 49 falls that hurt and 470 landings that didn't, levels 6 to
88), as Kawari plays them:

* **When**: the client's UpdatePositionHandler says when the player leaves the ground (MoveAnimationState
  LeavingCollision, 1, at byte 5; byte 4 is the animation type, byte 6 the jump state) and lands (EnteringCollision,
  2). The drop is the last height they stood at (taken while neither flag is set and the jump state is NoneOrFalling)
  less the landing's.
* **How much**: nothing below 10 yalms (9.99 never hurt, 10.0 did); then `10 + 0.228 × (drop − 10)²` % of the maximum
  HP (within 1.2 % of the 19 falls whose height could be measured, 4 % at most), rounded.
* **Never to 0 out of combat**: at least 1 HP is left (retail's 272 of 273 HP, 33,381 of 33,382 after the highest
  falls). In combat (Condition InCombat) a fall can kill (a long-time player's note, Bob, 2026-10-28).
* **What's sent**: ActorControl 45 `FallDamage` (the damage, 1) from the player to them and everyone around, then
  their StatusEffectList. No ActionEffect.
* Not hurt: mounted, dead, or in a gimmick jump.

**A mounted player isn't attacked** (since 2026-11-10, `npc_behavior`'s `mounted_players`): an enemy after one waits,
neither casting nor swinging, until they're off (retail's ambush ridden into, capture 1fac2416: the enemies came out
0.65 s after the walk-in, and the first acted 2.4 s after he got off, 5.3 s in; Kawari's hit him half a second after
they came, Bob's bundle 20261110-120801). Whether retail's enemies walk up to a mounted player meanwhile, and whether
one already fighting can be ridden away from, wasn't seen; Kawari's walk up.

## Paralysis (`server/paralysis.rs`)

Status 17, as retail played it out in Bob's Chronofoil captures (85 windows of it on players and NPCs, 2026-10-14 to
2026-11-01, from 7 to 30 s long, all with param 0):

* **When**: every 3 s while the actor has it (`ROLL_INTERVAL`), it may seize up: 290 procs in 593 three-second ticks,
  so one roll in two (`PROC_CHANCE`). Retail's procs were always a multiple of 3 s apart, at no fixed point after
  the status came; Kawari's first roll is 3 s after it does.
* **What's sent**: ActorControl 23 (`FloatingNumberVFX`) `17, 30, 0, actor` from the actor to everyone around, retail's
  values as they were.
* **What it stops**: whatever the actor is casting is cancelled, CancelCast `541, 1, action, 1` for a spell (players'
  Stone, Cure, Aero; Sylphie's; a chocobo's) and `540, 1, action, 1` for anything else (NPCs' weaponskills, a mount);
  a quest's progress bar `540, 4, bar, 1` ([Timing](timing.md#event-actions-progress-bars): Seeing Eye to Winged Eye's
  look-out, 0.9 s into the bar). An NPC acts on nothing for a second after (`LOCKOUT`: retail's next action came
  0.98 s or more after every proc); players' clients hold themselves back until the server's next conditions (below).
* **The player is let go a second later by their conditions.** A player's client seizes up by itself at a proc
  and stays so until the server next sends its conditions: retail sent them, unchanged, 0.98 s after every proc on the
  player (capture 44df02fc: all of them, cast cut short or not), which is why the next ActionRequest came 1.03-1.26 s
  after. Kawari never did (`release_players` now does, at the first tick a second after the proc). That was the freeze
  in Bob's six bundles of 2026-11-04 (20261104-150457 to -180355): a proc while the player was casting was let go by
  the Casting updates that followed anyway, but one that cut a cast short (conditions just before it, then nothing)
  left them unable to act or move for good, the status long gone. The fixes before it changed other things to
  retail's that weren't the cause:
  * the proc and CancelCast reach the player as ActorControl (the 2026.10.16 client hands ActorControl,
    ActorControlTarget and ActorControlSelf to the same function, 0x140B3A8C0, so it made no difference);
  * StatusEffectList and UpdateClassInfo after the proc (retail's 3 s tick, which the procs fall on);
  * the Casting condition, which retail sets right after the player's ActorCast and clears as the cast lands or is cut
    short (`action::set_casting`);
  * the status's param, 0 on retail (EffectResult and every StatusEffectList), 50 in Kawari's Paralyze and gaze
    scripts, read from the ActionEffect's status entry's second byte (`0E 32`), which isn't the param (the fourth is:
    Sprint's 30).
* **The paralysed player's own packets**, in retail's order: when the proc cut a cast short, Condition (Casting off)
  and CancelCast; the proc (ActorControl 23), both from the player; StatusEffectList and UpdateClassInfo; 0.98 s
  later Condition. Retail also sent ActorControl 1540 (`0, 3, an NPC`) with some procs, unknown, not sent.

Before 2026-11-01 paralysis did nothing (quest battle 17's gaze and the floating eye's gave it).

## Defeating enemies

When a battle NPC dies, everyone on its enmity list, and their party in the same instance, gets:

* **Quest credit**: each of their active quests' `onBNpcKill(base_id, layout_id, player)` (see [Quests](quests.md)).
* **EXP**: ParamGrow's `ScaledQuestXP` for the enemy's level (= 45 + 5 × level, exactly what retail gave; enemies
  above you give no more). Enemies **below** you give less (`kill_exp_percent`): all of it down to 2 levels below,
  then 10 % less a level (retail: 70 % at 4 below, 60 % at 5, about 40 % at 7). The level is the synced one when
  synced. From Bob's Chronofoil captures (141 kills, two characters).
* **Kill chains** for enemies at or above your level: +0, 20, 25, 30, 40, then 50 % (the sixth kill on), and the next
  kill has 100, 80, 60, 40, 20, then 10 s to keep it going (`exp_chain_bonus`; it was +20/25/30 in a flat 60 s).
  Each counting kill sends ActorControlSelf 12 `ExpChain` (its place from 0, the window in hundredths) before the EXP
  message, as retail. In duties there are no chains, and players synced down get no kill EXP at all (retail).
* **Rested EXP** (`rested_exp_part`): with enough in the pool the total is `floor(1.5 × base) − 1` and it uses up
  `floor((base + part) × 201600 / the bar's EXP)` seconds; a short pool gives `floor(pool × the bar's EXP / 201600 /
  3)` and empties. The chain's bonus is on the base, the rested part on top, the message showing both as one percent.
  Kill and gathering EXP only.
* **EXP packets:** the floating message's fourth value is the enemy (0xE0000000 for EXP from anything else); after a
  gain only SetEXP (ActorControlSelf 43: the EXP and rested pool) is sent, PlayerStats and the class info only on a
  level-up, whose message's fourth value is the level now (the synced one when synced).
* **Drops** (`resources/scripts/battle/Drops.lua`, by BNpcName): items from Garland Tools' data; chance and amount
  from recordings where known (anole 67 % ×3, ground squirrel 21 % ×2, trickster imp 11 %, counting Bob's 2026-10
  captures too), otherwise one 50 % roll (a guess). Central Shroud only so far. A drop is obtained directly with
  retail's **LootMessage** (fork opcode 190 on 2026.10.06, 32 bytes: kind 3, who, item, count → "You obtain ..."),
  then an ack (0xFFFFFFFF, 6). LootMessage's other kinds: 9 gil, 0/1 need/greed rolls and winners, 7 cast a lot, 2/8
  can't obtain.

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
  first status with Status.CanDispel), `effects:gaze()` (see Gazes), `effects:walk_to_caster(seconds, speed, pose)` (see Walked to the caster), `effects:heal_over_time_self(status, duration,
  tick)` (a regen on the caster: quest battle 17's boss's drain gives it Regen, 158, for 21 s at about 90 a tick); players' `player.is_player`, `hp`, `max_hp`, `mp`, `max_mp` for scripts.
* **Second Wind** is potency 500 below level 94 (it was 800).
* **Potencies measured from synced party members** (Bob's Chronofoil captures: plain hits by players synced to 31,
  36, 50 and 80; within a job, damage ratios between actions are potency ratios, anchored on actions no trait changes,
  which agreed within 0–3 %): every untraited action Kawari scripts already had retail's tooltip value; the ones a
  trait at 54–94 raises had their max-level value, now retail's for 1–50: Fast Blade 150, Riot Blade combo 260, Heavy
  Swing 150, Maim combo 250, Storm's Path/Eye combo 380, True Thrust 170, Vorpal Thrust combo 240, Disembowel combo
  200, Jump 250, Piercing Talon 150 (enhanced 230), Bootshine 190, True Strike 270, Snap Punch 240, Twin Snakes 340,
  Demolish 280, Dragon Kick 250, Spinning Edge 180, Gust Slash combo 260, Aeolian Edge combo 280, Throwing Dagger
  120, Scholar's Ruin II 140, Art of War 150, Slice 260. Positionals measured (not done): Snap Punch flank, Demolish
  and Aeolian Edge rear +60, Trick Attack rear +100; Monk's Raptor's Fury +200, Coeurl's Fury +150; Astral Fire III ×1.8
  fire and ×0.7 ice; direct hit ×1.25, critical about ×1.55 at sync 50. Out-of-combo potencies of combo steps
  weren't measurable (almost no such hits).
* **Statuses checked against retail** (every status any action applied in the captures, with the scripts run under
  lupa): IDs and durations agreed for nearly every scripted action. Fixed: Sheltron is status 1856 (not 728);
  Temperance's damage reduction is the party's 1873 (1872, the White Mage's, is +20 % healing magic, not
  implemented); Rampart has no healing bonus below level 94; Firestarter and Thunderhead have no time limit, and
  Thunderhead is only granted when it isn't up (never by Umbral Soul); Shield Samba goes to the party; Cover's param
  20; True Grit's Blind 7 s and Acorn Armageddon's Heavy 15 s. Surging Tempest's refresh adds 30 s to the time left
  (up to 60) and eating again adds the food's time to Well Fed's (up to twice it), as retail; scripts can read a
  status's time left (`player:effect_time_left(id)`). The self-status effect (kind 15) carries 0x80 in its
  last byte, the target's EffectResult names the target, and ActorControl 20's source is left empty, as retail.
* **Raise and Resurrection** offer the dead target a revive (status 148, param 8, 60 s; the effect's second byte 20).
  Accepting it (client trigger 200 with 5) brings them back beside their raiser (the revive offer's source, where they
  stand as it's accepted: retail put the raised player within a yalm of the raiser in every one of 44 raises in Bob's
  captures, up to 21 yalms from where they died; Kawari had them get up where they lay, Bob's bundle of 2026-10-25) with 20 % of their HP and MP, Weakness
  (43, 100 s; Brink of Death 44 if already weakened) and status 418 (5 s), through a warp with the raise's effect (181,
  warp type 5); returning to the starting point (8) gives full HP and MP and takes Weakness and Brink away. From
  Bob's captures (19 raises, 7 returns). Retail also lowers main stats by about 26 % under Weakness and 50 % under
  Brink (not done), stands the player up 3 s after the warp, and keeps Weakness through a death.
* **Aetherflow** (Arcanist/Summoner, 2026-10-22): Energy Drain fills it to two stacks (`gain_aetherflow`), Fester and
  Painflare spend one (`spend_aetherflow`); the client won't use Fester without a stack, so before this it couldn't be
  cast at all (Bob's 2026-10-22 bundle). It's the low two bits of the ninth byte of ActorGauge's data (retail: 0x02
  after each Energy Drain, 0x01 after the Fester after it, with Summoner's attunement flags in the other bits; Bob's
  Chronofoil capture of 2026-10-21), `job_gauge.rs`.
* **Carbuncle** (`server/pet.rs`): "Away" (PetAction 1, client trigger 1800) dismisses it as retail does: the owner
  gets ActorControlSelf 269 with the pet's BNpcName and 1, SetupPet with no pet, SetPetParameters 0 and 268 with the
  name and 0; the pet gets Kill and SetMode dead, DeadFadeOut 2.1 s later and is deleted 1.75 s after that (Bob's
  capture of 2026-10-21, three dismissals). Following its owner it walks straight on where the navmesh doesn't reach
  (as party bots do), and one more than 50 yalms behind (`PET_CATCH_UP_DISTANCE`, a guess: retail's only fell that far
  behind after teleports) is put beside its owner with an ActorSetPos, as retail puts it 1.9 yalms from its owner after
  a teleport. Other pet actions (Heel and the rest) are still only logged.
* **Job gauges** (`server/job_gauge.rs`, since 2026-11-10 beyond the Lilies and Aetherflow): counters the action
  scripts change by name (`effects:gauge_add("beast", 10)`, `effects:gauge_set("element", 3)`) and read
  (`player:gauge("beast")`), sent to the client in ActorGauge at FFXIVClientStructs' offsets (WarriorGauge.BeastGauge
  0x08, MonkGauge.Chakra 0x08, BlackMageGauge.ElementStance 0x0A and UmbralHearts 0x0B). Party bots keep their own
  (`PartyBot::gauge`, reset at a wipe).
  * **Warrior's Beast Gauge** (Global.lua `warGainBeast`/`warSpendBeast`): from The Beast Within (35) Maim's combo 10,
    Storm's Path's 20, Storm's Eye's 10, Mythril Tempest's 20 from 74, Infuriate 50; Inner Beast and Steel Cyclone
    spend 50, nothing under Inner Release. Infuriate gives Nascent Chaos only from 72 (it gave it at 50).
  * **Monk's chakra:** Steeled and Inspirited Meditation open one; from Deep Meditation (38) a critical weaponskill
    opens one (`server/action.rs`, `DEEP_MEDITATION_LEVEL`); Steel Peak and Howling Fist spend five. Perfect Balance's
    three stacks go one a weaponskill (`mnkUsePerfectBalance`; they never went before).
  * **Black Mage's Astral Fire and Umbral Ice** (Global.lua `blm*`, the official job guide's numbers for 7.x): Fire
    spells 40/60/80 % stronger in Astral Fire I/II/III and twice the MP without Umbral Hearts, free in Umbral Ice and
    10 % weaker a stack there; Ice spells free in Umbral Ice, 2,500/5,000/10,000 MP back on landing one there, 10 %
    weaker a stack in Astral Fire; no MP over time in Astral Fire (players and bots, `server/mod.rs`); Fire and Fire II
    a stack of Astral Fire (or Umbral Ice gone), Blizzard and Blizzard II the other way, Fire III and Flare Astral Fire
    III, Blizzard III Umbral Ice III, Transpose one of the other, Umbral Soul one more Umbral Ice and an Umbral Heart
    from 58, Freeze three hearts from 58; one stack at most before 20, two before 35. Thunderhead comes with a change of
    element (or one from none), as the client's tooltip says, where it came with every Fire and Blizzard.
* **Not there:** pets other than Carbuncle (no egis, no Eos, no
  demi-summons: their attacks come with the Summoner's spells), positionals, movement actions (Thunderclap,
  Rescue...), barriers absorbing damage, Bloodbath's lifesteal, Peloton for the party. See
  [Known Issues](known-issues.md#combat).

**Combat items** (`items/Items.lua`): potions (ItemAction 847: the % of max HP up to the cap), ethers (848, MP),
elixirs (849, their caps), status removal (842), food (844–846: Well Fed 48 with the ItemFood row and duration, no stat
bonus yet).

## The ARR jobs, levels 51–100

Since 2026-11-10 every action of Paladin, Warrior, Monk, Dragoon, Bard, Ninja, White Mage, Black Mage, Summoner and
Scholar from 51 to 100 has a script (about 230 new, from the official job guide's 7.x tooltips at level 100; each
script's header says so), and their job gauges are played out:

* **Gauge fields** (`lua::GaugeField`, 26 counters an array in `JobGauge::fields`, each with its range): beast, chakra
  (0-10), element, umbral_hearts, polyglot, paradox, astral_soul, enochian_timer, oath, opo_fury, raptor_fury,
  coeurl_fury, beast_chakra (three 2-bit slots), nadi, lotd (tenths of a second), firstminds_focus, repertoire,
  soul_voice, coda, ninki, kazematoi, mudras (three 2-bit slots), attunement, attunement_type, arcana, fairy; plus
  Aetherflow (`gain_aetherflow(n)`: two for Energy Drain, three for the Scholar's Aetherflow and Dissipation) and the
  Lilies. The server ticks two of them (`JobGauge::tick`, players and party bots, `server/mod.rs`): Life of the Dragon
  runs down, and from 70 a Black Mage in Astral Fire or Umbral Ice gets a Polyglot every 30 s (one at most at 70, two
  from 80, three from 98).
* **ActorGauge layout** (`job_gauge.rs`'s `to_packet_data`, at FFXIVClientStructs' offsets): data[0] is offset 0x07
  for the Paladin (Oath at data[1], 0x64 at 100 in Bob's captures) and the Summoner (AetherFlags at data[8]), 0x08 for
  the White Mage (Lilies); the others are assumed to be 0x07 like the Paladin's and Summoner's (`at()`), not yet seen.
* **Guaranteed critical and direct hits** (`effects:guarantee_critical()`, `guarantee_direct_hit()`): the hit's rolls
  are 0 (`server/action.rs`): Opo-opo form's Bootshine/Leaping Opo, Life Surge, Recitation's heals...
* **Paladin**: Oath 5 a landed auto-attack from 35 (`pldGainOath`, both auto-attack scripts), Sheltron, Holy Sheltron,
  Intervention 50; Fight or Flight gives Goring Blade Ready from 54, Requiescat / Imperator the Confiteor chain and
  Divine Might, Royal Authority the Atonement chain (`pldMagicPotency`).
* **Warrior**: Berserk / Inner Release make the next three weaponskills sure criticals and direct hits (`warBerserk`);
  Inner Release's Primal Rend, Primal Wrath, Primal Ruination; Infuriate's Nascent Chaos turns Fell Cleave and Decimate
  into Inner Chaos and Chaotic Cyclone.
* **Monk**: the Furies (Dragon Kick, Twin Snakes, Demolish give theirs, Bootshine, True Strike, Snap Punch spend them),
  one form at a time (`mnkShift`), the Beast Chakra under Perfect Balance from 60 (`mnkBeastChakra`), Masterful Blitz
  (three of one Elixir Field / Burst and the Lunar Nadi, three different Flint Strike / Rising Phoenix and the Solar,
  two kinds Celestial Revolution, both Nadi Tornado Kick / Phantom Rush), chakra on four critical weaponskills in five
  from 38 and every one from 74 (Deep Meditation II), Fire's and Wind's Reply after their Riddles.
* **Dragoon**: Life of the Dragon (Geirskogul from 70, 20 s, 15 % more damage: `drgDamage`), Nastrond and Stardiver in
  it, Life Surge's sure critical (`drgWeaponskill`), Dive Ready, Dragon's Flight, Starcross Ready, Firstminds' Focus
  for Wyrmwind Thrust.
* **Bard**: the songs (`brdStartSong`: one at a time, Repertoire from the weaponskills while one lasts with Soul Voice 5
  from 80, the Codas from 90), Pitch Perfect at the Minuet's Repertoire, Apex Arrow's Soul Voice and Blast Arrow,
  Hawk's Eye from Burst Shot (35 %; Burst Shot was upstream's stub without it), Barrage's Resonant Arrow, Radiant
  Finale's Radiant Encore.
* **Ninja**: the mudras (`ninMudra`: Ten 1, Chi 2, Jin 3, the IDs 18805-18807 retail's client sends for the second
  and third too), Ninjutsu casts what they make (`ninNinjutsuOf`: Fuma Shuriken, Katon, Raiton, Hyoton, Huton, Doton,
  Suiton, Goka Mekkyaku and Hyosho Ranryu under Kassatsu, Rabbit Medium for a repeated mudra), Shadow Walker for
  Trick Attack / Kunai's Bane and Meisui, Raiju after Raiton, Ninki from 62 (Bunshin, Bhavacakra, Zesho Meppo...),
  Kazematoi.
* **Black Mage**: Enochian (56: 5 % more in either element, up to 27 % from 96), Polyglot (Foul, Xenoglossy,
  Amplifier), Paradox (switching from Astral Fire III or from Umbral Ice III with three hearts, from 90), Astral Soul
  (Fire IV, Flare three) for Flare Star, Fire IV / Despair / Blizzard IV; Manafont also gives Astral Fire III, three
  Umbral Hearts and Paradox, as the job guide says.
* **Summoner**: the arcana (Aethercharge, and from 58 the demi-summons' trance: `arcana`), the summons (`smnSummon`:
  Ruby / Topaz / Emerald Carbuncle, Ifrit, Titan, Garuda, their IIs from 90) attuning it, the gem spells retail's
  client sends (Ruby/Topaz/Emerald Ruin I-III, Rite, Outburst, Disaster, Catastrophe, 25808-25834: `smnGem`), the
  Favors (Mountain Buster, Slipstream, Crimson Cyclone - Crimson Strike), Further Ruin for Ruin IV, Searing Light's
  Ruby's Glimmer, Dreadwyrm / Firebird / Lightwyrm Trance's spells with the demi's own attack (`smnDemiHit`: no pets).
* **Scholar**: Aetherflow three stacks, spent by Energy Drain, Lustrate, Sacred Soil, Indomitability and
  Excogitation (none under Recitation, whose heals are critical: `schSpendAetherflow`), 10 Faerie Gauge each from 70
  (Aetherpact spends it), Dissipation, Chain Stratagem's Impact Imminent for Baneful Impaction, Seraphism's
  Manifestation and Accession. The fairy and Seraph aren't there: Fey Blessing and Consolation are the Scholar's own.
* **White Mage**: Liturgy of the Bell ended early (28509) heals 200 a stack left.

How they were checked: run under lupa against each job's rotation (a harness running the real scripts with a mock
EffectsBuilder, every level band and one or three targets, no errors) and against retail's uses in Bob's captures
where there were some (Paladin, Warrior, Monk, Dragoon, Black Mage, Summoner, Scholar and White Mage at 80-89).

## The jobs from Heavensward on, levels 1–100

Since 2026-11-10 Dark Knight, Gunbreaker, Machinist, Samurai, Dancer, Reaper, Viper, Red Mage, Pictomancer,
Astrologian and Sage have every action scripted (436 new scripts and 18 of upstream's stubs replaced, from the official job guide's 7.x text: level-100 numbers,
with the lower ones its traits list below their levels through `tierPotency`, Maim and Mend and Increased Action Damage
by level through `levelPotency`; status IDs from Boss Mod's lists and the statuses the actions put on in Bob's captures).
Each job's helpers are in Global.lua (`drk*`, `gnb*`, `mch*`, `sam*`, `dnc*`, `rpr*`, `vpr*`, `rdm*`, `pct*`, `ast*`,
`sge*`), and its gauge is played out:

* **Gauge fields** (`lua::GaugeField`, 64 now): blood, darkside (tenths of a second, run down by `JobGauge::tick`),
  dark_arts; cartridges (two, three from 88, six under Bloodfest); heat, battery; feathers, esprit, dance_steps (three
  bits a step), step_index; kenki, sen (1 Setsu, 2 Getsu, 4 Ka), meditation, kaeshi; soul, shroud, lemure, void;
  offerings, coils, tribute; white_mana, black_mana, mana_stacks; palette, white_paint, canvas (the motifs: 1 Pom, 2
  Wing, 4 Claw, 8 Maw, 16 hammer, 32 starry sky), creature (depictions 1-4, portraits 16 Moogle, 32 Madeen); ast_cards
  (three cards, four bits each), ast_arcana, ast_draw; addersgall (one every 20 s from 45, `tick`), addersgall_timer,
  addersting, eukrasia. Three are the server's only, not shown: shadow_hits (the Dark Knight's simulacrum), pet_hits
  and pet_potency (the Machinist's turret and queen).
* **ActorGauge** at FFXIVClientStructs' offsets less 7 (`at`), which the Dark Knight's packets in Bob's captures
  confirm (DarksideTimer, 0x0A, at data[3..5]: 0x7530 after an Edge of Darkness) as the Black Mage's do (ElementStance,
  0x0A, data[3]: 0xFF in Umbral Ice I).
* **Pets** aren't on the server, so their attacks come with the job's own weaponskills: the Dark Knight's Living
  Shadow (six hits, 420 each, Shadowbringer's 570 and Disesteem's 620 among them), the Machinist's Rook Autoturret
  (four, 84 a hit at 50 Battery) and Automaton Queen (six, 262 at 50 Battery, 197 before Crowned Collider at 86), twice
  that at 100 Battery; Rook / Queen Overdrive lands what's left at once.
* **Instant casts** (`action.rs`'s `INSTANT_CASTS`): Dualcast (the Red Mage's spells give it, `rdmDualcast`, and spend
  it), Acceleration for Verthunder, Veraero (III), Scatter and Impact, Enhanced Harpe, Rainbow Bright for Rainbow Drip,
  Lightspeed. The server's cast bar was the full cast before.
* **Wildfire**: 240 for each of the Machinist's next six weaponskills, as they land (its end isn't marked on the
  server).
* **The Dancer's dance**: Standard / Technical Step draws its steps at random into the gauge, each step is done if
  it's the one due, and the finish the client sends (16191-16192, 16193-16196 for one to four steps) hits for its
  step count; Technical Finish's party buff comes with 33215-33218, which retail's client sends straight after the
  finish (always 33218 after 16196 in Bob's captures).
* **Retail's client IDs**: Enchanted Riposte, Zwerchhau and Redoublement under Magicked Swordplay as 45960-45962 (Bob's
  captures, Red Mages at 80-89, four each, as many as Manafication allows; scripted as the plain ones).
* **Combos** (`RotationState::last_combo`, party bots): only combo steps (an action with ActionCombo, or one another's
  ActionCombo names) carry a bot's combo on or break it, as players' already did (`takes_part_in_combos`): Gnashing
  Fang, Burst Strike, Sonic Break and the like between leave it be. Retail's Gunbreakers at 80-89 used Keen Edge 279
  times, Brutal Shell 268, Solid Barrel 253 with 123 Gnashing Fangs among them; the bots broke the combo at each.
* **Buffs and mitigation** (`modifiers.rs`): No Mercy, Fugetsu, Hunter's Instinct, Arcane Circle, Standard and
  Technical Finish, Embolden, Divination, The Balance and The Spear, Starry Muse (and Brotherhood, Riddle of Fire,
  Searing Light) on damage dealt; Death's Design on the target; Shadow Wall, Dark Mind, Oblation, Dark Missionary,
  Camouflage, Nebula, Great Nebula, Heart of Light / Stone / Corundum, Tactician, Shield Samba, Collective
  Unconscious, The Bole, Sun Sign, Kerachole, Taurochole, Holos, Expedient, Intervention, Passage of Arms and Divine
  Veil on damage taken; Superbolide's invulnerability, Living Dead's HP floor; The Arrow, Krasis, Physis II and
  Protraction on healing.
* **Raises**: Ascend, Egeiro and Verraise offer the revive as Raise does.

How they were checked: every script run under lupa at levels 1, 50, 80 and 100 with empty and full gauges (no errors),
and each job's rotation simulated with the real scripts at every band against one and three targets (every GCD used;
the Red Mage's casts instant under Dualcast). Against retail's players at 80-89 in Bob's captures: the Dark Knight's
Hard Slash, Syphon Strike, Souleater and Bloodspiller about 6 a minute each (retail 216, 211, 193, 215 uses), the
Gunbreaker's Keen Edge - Brutal Shell - Solid Barrel 4.7 / 4.3 / 4.3 with Gnashing Fang 2 (279 / 268 / 253 / 123),
the Samurai's Hakaze 6, Shinten 6.7, Gekko 3, Yukikaze 3, Kasha 2.7, Midare 2.3 (85, 67, 44, 39, 42, 34).

## Adding a monster or skill

1. Record it (Kawari Recorder → Monsters tab: level, HP per level, skills, damage to players at your level).
2. Timeline: `resources/timelines/<Name>_<BNpcBase>.json` with its hitbox, auto-attack, `damage_to_players`,
   timepoints.
3. Skill script: `resources/scripts/actions/<id / 100>/<Name>_<id>.lua`, potency relative to its auto-attack.
4. Drops: `Drops.lua` by BNpcName.
5. Fixed spawns: the zone's drop-in.
